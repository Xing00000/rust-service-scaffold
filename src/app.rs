// src/app.rs
use crate::infrastructure::telemetry;
use crate::infrastructure::web::handlers;
use crate::{config::Config, infrastructure::web::metrics};
use axum::{middleware, routing::get, Router};
use prometheus::Registry;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
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
            config: Arc::new(config.clone()),
            registry: Arc::new(registry),
        };

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
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));

        let untracked_routes = Router::new().route("/metrics", get(handlers::metrics_handler));

        // ✅ 將兩個 Router 合併，並應用最終的 state
        let router = Router::new()
            .merge(tracked_routes)
            .merge(untracked_routes)
            .with_state(app_state);

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
