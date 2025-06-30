use crate::config::Config;
use crate::state::AppState;
use axum::{middleware, routing::get, Router};
use hyper::header::{HeaderName, HeaderValue};
use infra_db_postgres::user_repo::PostgresUserRepository;
use infra_telemetry::{config::TelemetryConfig, metrics::Metrics, metrics_layer, telemetry};
use pres_web_axum::handlers;
use tower::ServiceBuilder;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

pub mod shutdown; // Declare the shutdown module
use shutdown::{OpenTelemetryShutdownHook, ShutdownHooks, UserRepositoryShutdownHook};

pub struct Application {
    router: Router,
    listener: TcpListener,
    shutdown_hooks: Arc<ShutdownHooks>, // Store hooks for shutdown_signal
}

// No complex Stack return type needed with this approach
// Function to apply common Tower layers to a router
fn apply_common_layers(
    router: Router,
    app_config: &Config, // Changed from &AppState to &Config
    metrics: Arc<Metrics>,
) -> Router {
    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(app_config.rate_limit_per_second)
            .burst_size(app_config.rate_limit_burst_size)
            .finish()
            .unwrap(),
    );

    ServiceBuilder::new()
        .layer(axum::extract::Extension(metrics.clone()))
        .layer(middleware::from_fn(
            pres_web_axum::middleware::telemetry_middleware::axum_metrics_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(GovernorLayer {
            config: governor_config,
        })
}


impl Application {
    pub async fn build(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let telemetry_cfg: TelemetryConfig = TelemetryConfig {
            otel_service_name: config.otel_service_name.clone(),
            otel_exporter_otlp_endpoint: config.otel_exporter_otlp_endpoint.clone(),
            prometheus_path: "/metrics".to_string(),
            log_level: config.log_level.clone(),
        };
        let registry = telemetry::init_telemetry(&telemetry_cfg)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        std::panic::set_hook(Box::new(telemetry::panic_hook));
        let user_repo_impl =
            PostgresUserRepository::new(&config.database_url, config.db_max_conn).await?;
        let user_repo: Arc<dyn application::ports::UserRepository> = Arc::new(user_repo_impl);

        // Create shutdown hooks
        let mut hooks: ShutdownHooks = Vec::new();
        hooks.push(Box::new(OpenTelemetryShutdownHook));
        hooks.push(Box::new(UserRepositoryShutdownHook::new(user_repo.clone())));
        let shutdown_hooks = Arc::new(hooks);

        let app_state = AppState {
            config: Arc::new(config.clone()), // Clone config for app_state
            registry: Arc::new(registry),
            user_repo: user_repo.clone(),
            shutdown_hooks: shutdown_hooks.clone(), // Add to AppState
        };

        // Configure Governor for rate limiting using values from Config
        let governor_config = Arc::new(
            GovernorConfigBuilder::default()
                .per_second(config.rate_limit_per_second)
                .burst_size(config.rate_limit_burst_size)
                .finish()
                .unwrap(),
        );
        let metrics = Arc::new(Metrics::new());
        // Note: app_state.config is Arc<Config>, apply_common_layers expects &Config
        // So we pass &app_state.config (which is &Arc<Config>) and deref it inside, or pass config directly.
        // Let's pass 'config' (the original one before it's moved into Arc for app_state)
        // or clone it if necessary. Since app_state.config is Arc, we can pass a reference to the inner Config.
        // For simplicity, using app_state.config.as_ref() or simply app_state.config for Arc fields.
        // The function signature is app_config: &Config.

        let tracked_routes = Router::new()
            .route("/", get(handlers::main_handler::<AppState>))
            .route("/test_error", get(handlers::test_error_handler))
            .route("/test_panic", get(handlers::panic_handler))
            .route("/healthz/live", get(handlers::live_handler))
            .route("/healthz/ready", get(handlers::ready_handler))
            .route("/info", get(handlers::info_handler))
            // User routes
            .route("/users", axum::routing::post(handlers::create_user_handler::<AppState>))
            .route("/users/:id", get(handlers::get_user_handler::<AppState>));

        let untracked_routes =
            Router::new().route("/metrics", get(handlers::metrics_handler::<AppState>));

        // ✅ 將兩個 Router 合併，並應用最終的 state
        let base_router = Router::new()
            .merge(tracked_routes)
            .merge(untracked_routes);

        // Apply common layers using the new function
        // Pass config.as_ref() if config is Arc<Config> in current scope,
        // or just &config if it's Config.
        // AppState has Arc<Config>, so app_state.config.as_ref() or &*app_state.config
        let router_with_common_layers = apply_common_layers(base_router, &app_state.config, metrics);


        // Apply HTTP headers from config
        let mut router = router_with_common_layers; // Start with the router that has common layers
        if let Some(headers_config) = &app_state.config.http_headers {
            for header_config in headers_config {
                let header_name = HeaderName::from_bytes(header_config.name.as_bytes())
                    .unwrap_or_else(|_| {
                        panic!("Invalid header name in config: {}", header_config.name)
                    });
                let header_value =
                    HeaderValue::from_str(&header_config.value).unwrap_or_else(|_| {
                        panic!(
                            "Invalid header value for {}: {}",
                            header_config.name, header_config.value
                        )
                    });
                router = router.layer(SetResponseHeaderLayer::if_not_present(
                    header_name.clone(), // Clone since it's used in the error message too
                    header_value,
                ));
            }
        }

        let router = router.with_state(app_state);

        let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("Listening on {}", addr);

        Ok(Application { router, listener, shutdown_hooks })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        tracing::info!("Application started. Press Ctrl+C to shut down.");
        axum::serve(
            self.listener,
            self.router
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal(self.shutdown_hooks)) // Pass hooks
        .await
    }
}

// ✅ [關鍵新增] 添加一個異步函數來監聽操作系統的關閉信號
async fn shutdown_signal(hooks: Arc<ShutdownHooks>) { // Accept hooks
    // 創建一個 Future 來處理 Ctrl+C 信號
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // 僅在 Unix 系統上創建一個 Future 來處理 TERM 信號
    // Kubernetes 等容器編排平台會發送 SIGTERM 來終止 Pod
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    // 在 Windows 上，我們只等待 Ctrl+C
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // 使用 tokio::select! 宏來等待任何一個信號
    // `tokio::select!` 會在第一個完成的 Future 上停止等待，並取消其他的 Future
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::warn!("Signal received, starting graceful shutdown...");

    // Execute all shutdown hooks
    for hook in hooks.iter() {
        hook.shutdown().await;
    }
    tracing::info!("All resources shut down gracefully.");
}
