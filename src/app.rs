// src/app.rs
use crate::config::Config;
use crate::infrastructure::telemetry;
use crate::infrastructure::web::handlers;
use axum::{routing::get, Router};
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
}

pub struct Application {
    router: Router,
    listener: TcpListener,
}

impl Application {
    pub async fn build(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        telemetry::init_telemetry(&config)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        std::panic::set_hook(Box::new(telemetry::panic_hook));

        let app_state = AppState {
            config: Arc::new(config.clone()),
        };

        let router = Router::new()
            .route("/", get(handlers::main_handler))
            .route("/test_error", get(handlers::test_error_handler))
            .route("/test_panic", get(handlers::panic_handler))
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(TraceLayer::new_for_http())
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .with_state(app_state);

        let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("Listening on {}", addr);

        Ok(Application { router, listener })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        axum::serve(self.listener, self.router.into_make_service()).await
    }
}
