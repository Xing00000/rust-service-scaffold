use axum::{
    extract::{Extension, Query},
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use std::net::SocketAddr;
use tower_http::{
    request_id::{PropagateRequestIdLayer, RequestId},
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use serde::Deserialize;

use crate::logging;
use crate::telemetry;
use crate::handlers; // For panic_handler
use crate::error::AppError;

// OpenTelemetry imports for handler context
use opentelemetry::{Context as OtelContext, trace::TraceContextExt};

pub struct Application {
    router: Router,
    listener: TcpListener,
}

impl Application {
    pub async fn build() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize OpenTelemetry Tracing and Logging
        let otel_tracer_result = telemetry::init_tracer();

        match otel_tracer_result {
            Ok(tracer) => {
                logging::init_subscriber(Some(tracer));
                tracing::info!("OpenTelemetry tracer initialized successfully for Application.");
            }
            Err(e) => {
                logging::init_subscriber(None);
                tracing::warn!("Failed to initialize OpenTelemetry tracer for Application: {:?}. Proceeding without OTel tracing.", e);
            }
        }

        // Set the custom panic hook
        std::panic::set_hook(Box::new(telemetry::panic_hook));

        let router = Router::new()
            .route("/", get(handler))
            .route("/test_error", get(test_error_handler))
            .route("/test_panic", get(handlers::panic_handler)) // From crate::handlers
            // Propagate X-Request-ID header or generate a new one
            .layer(PropagateRequestIdLayer::x_request_id())
            // TraceLayer for logging HTTP requests
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &axum::http::Request<_>| {
                        let request_id = request
                            .extensions()
                            .get::<RequestId>()
                            .and_then(|id| id.header_value().to_str().ok())
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "unknown".into());
                        tracing::span!(
                            Level::INFO,
                            "http_request",
                            method = %request.method(),
                            uri = %request.uri(),
                            version = ?request.version(),
                            request_id = %request_id,
                        )
                    })
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            );

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        let listener = TcpListener::bind(addr).await?;

        Ok(Application { router, listener })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        tracing::info!("listening on {}", self.listener.local_addr()?);
        axum::serve(self.listener, self.router.into_make_service()).await
    }
}

// Moved from main.rs
#[derive(Deserialize)]
struct HandlerParams {
    make_error: Option<bool>,
}

// Moved from main.rs
async fn handler(
    Extension(request_id_extension): Extension<RequestId>,
    Query(params): Query<HandlerParams>,
) -> Result<String, AppError> {
    let request_id = request_id_extension
        .header_value()
        .to_str()
        .unwrap_or("unknown")
        .to_string();

    if params.make_error.unwrap_or(false) {
        tracing::info!("Simulating an error for request_id: {}", request_id);
        return Err(AppError::BadRequest(format!("Triggered error for request_id: {}", request_id)));
    }

    let current_otel_cx = OtelContext::current();
    let _current_otel_span = current_otel_cx.span();

    let greeting_message = "Hello, World!";
    let response_body = {
        let _custom_work_span_guard = tracing::info_span!(
            "custom_work_in_handler",
            service_operation = "generate_greeting",
            request_id = %request_id
        ).entered();
        tracing::info!("Performing custom work: generating greeting message for request_id: {}", request_id);
        format!("{} Request ID: {}", greeting_message, request_id)
    };

    tracing::info!("Request processing finished for request_id: {}", request_id);
    Ok(response_body)
}

// Moved from main.rs
async fn test_error_handler() -> Result<&'static str, AppError> {
    Err(AppError::InternalServerError)
}
