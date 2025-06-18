use axum::{
    extract::Extension,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    request_id::{PropagateRequestIdLayer, RequestId},
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::Level;

// Use the logging module from the library crate
use axum_logging_service::logging;

#[tokio::main]
async fn main() {
    logging::init_subscriber(); // Initialize logging

    let app = Router::new()
        .route("/", get(handler))
        // Layer to propagate X-Request-ID header or generate a new one
        .layer(PropagateRequestIdLayer::x_request_id())
        // Layer to include the request_id in tracing spans
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    // Attempt to get RequestId from extensions
                    // This is where MakeRequestUuid (if used directly) or PropagateRequestIdLayer stores it
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
                        request_id = %request_id, // Add request_id to the span
                    )
                })
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        // Layer to apply the MakeRequestUuid logic
        // This should ideally be outside TraceLayer if TraceLayer is to pick it up from extensions
        // For tower-http 0.4, MakeRequestUuid is a service, not a layer.
        // We need MakeRequestUuidLayer for it to be a layer.
        // Let's assume tower-http 0.4 style: MakeRequestUuid is often part of the service builder.
        // The PropagateRequestIdLayer handles setting the extension.
        ;


    // The MakeRequestUuid should be set on the service builder.
    // Axum's Server::bind takes a MakeService. We need to wrap it.
    // However, for simplicity with layers, PropagateRequestIdLayer is often sufficient if
    // we ensure it runs before TraceLayer. It reads X-Request-ID or generates one.

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(Extension(request_id_extension): Extension<RequestId>) -> String {
    // The RequestId extension should be available if PropagateRequestIdLayer is used.
    let request_id = request_id_extension
        .header_value()
        .to_str()
        .unwrap_or("unknown")
        .to_string();

    // Log with the request_id explicitly for testing, though it should be in the span already
    tracing::info!(request_id = %request_id, "Handler processing request");
    tracing::debug!(request_id = %request_id, "Detailed handler action");

    format!("Hello, World! Request ID: {}", request_id)
}
