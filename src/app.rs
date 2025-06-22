// src/app.rs
use crate::infrastructure::telemetry;
use crate::infrastructure::web::handlers;
// Import the auth_middleware
use crate::infrastructure::web::middleware::auth_middleware;
use crate::{config::Config, infrastructure::web::metrics};
use axum::{middleware, routing::get, Router};
use hyper::header::{HeaderName, HeaderValue};
use prometheus::Registry;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub registry: Arc<Registry>,
}

pub struct Application {
    router: Router,
    listener: TcpListener,
}

impl Application {
    pub async fn build(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let registry = telemetry::init_telemetry(&config)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        std::panic::set_hook(Box::new(telemetry::panic_hook));

        let app_state = AppState {
            config: Arc::new(config.clone()), // Clone config for app_state
            registry: Arc::new(registry),
        };

        // Configure Governor for rate limiting using values from Config
        let governor_config = Arc::new(
            GovernorConfigBuilder::default()
                .per_second(config.rate_limit_per_second)
                .burst_size(config.rate_limit_burst_size)
                .finish()
                .unwrap(),
        );

        let tracked_routes = Router::new()
            .route("/", get(handlers::main_handler))
            .route("/test_error", get(handlers::test_error_handler))
            .route("/test_panic", get(handlers::panic_handler))
            .route("/healthz/live", get(handlers::live_handler))
            .route("/healthz/ready", get(handlers::ready_handler))
            .route("/info", get(handlers::info_handler))
            .layer(middleware::from_fn(metrics::track_metrics))
            .layer(TraceLayer::new_for_http())
            .layer(PropagateRequestIdLayer::x_request_id())
            // IMPORTANT: SetRequestIdLayer must be before GovernorLayer
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            // Add GovernorLayer for rate limiting
            .layer(GovernorLayer {
                config: governor_config,
            })
            // Add our new auth_middleware
            // Note: The auth_middleware needs to be adapted if it doesn't match `from_fn`'s expected signature directly.
            // Our current auth_middleware expects HeaderMap, Request, Next.
            // `from_fn` expects a function that takes Request, Next.
            // We'll need a wrapper or to adjust auth_middleware.
            // For now, let's assume a wrapper `auth_middleware_wrapper` similar to tests or adjust it.
            // Let's create a simple inline wrapper for now, or adjust auth_middleware to fit.
            // Simpler: adjust auth_middleware or use from_fn_with_state if state is needed early.
            // Given auth_middleware's current signature: `async fn auth_middleware(headers: HeaderMap, request: Request, next: Next)`
            // it's not directly compatible with `middleware::from_fn`.
            // Let's use `middleware::from_fn_with_state` if we needed AppState in middleware,
            // or a simple wrapper.
            // The simplest way is to modify auth_middleware to take `Request, Next` and extract headers inside.
            // Let's go back and modify `auth.rs` for this.
            // For now, I will add it assuming it's compatible or will be made compatible.
            // This highlights a potential refinement for auth.rs or the need for a wrapper here.
            // Let's assume we will adjust auth_middleware to be: `async fn auth_middleware(request: Request, next: Next)`
            .layer(middleware::from_fn(auth_middleware)); // This now expects auth_middleware to have the compatible signature

        let untracked_routes = Router::new().route("/metrics", get(handlers::metrics_handler));

        // ✅ 將兩個 Router 合併，並應用最終的 state
        let mut router = Router::new().merge(tracked_routes).merge(untracked_routes);

        // Apply HTTP headers from config
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

        Ok(Application { router, listener })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        tracing::info!("Application started. Press Ctrl+C to shut down.");
        axum::serve(self.listener, self.router.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
    }
}

// ✅ [關鍵新增] 添加一個異步函數來監聽操作系統的關閉信號
async fn shutdown_signal() {
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
}
