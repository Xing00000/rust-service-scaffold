use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_subscriber::fmt::{self, Layer as FmtLayer}; // Removed format::JsonFields as it's not directly used
use tracing_opentelemetry::OpenTelemetryLayer;
use opentelemetry::global;

pub fn init_subscriber() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info")); // Default to info level, can be overridden by RUST_LOG

    // Create a formatting layer (e.g., JSON)
    let formatter = FmtLayer::default()
        .json() // Output logs in JSON format
        .with_current_span(true) // Include current span info
        .with_span_list(true) // Include span list (shows parent spans)
        .with_timer(fmt::time::rfc_3339()) // Use RFC 3339 timestamps
        .with_thread_ids(true) // Include thread IDs
        .with_thread_names(true); // Include thread names

    // Get the global tracer for OpenTelemetry layer
    // The name here is for the tracing instrumentation, not the service name itself.
    let tracer = global::tracer("axum-logging-service/tracing-integration");
    let otel_layer = OpenTelemetryLayer::new(tracer);

    Registry::default()
        .with(env_filter)
        .with(formatter) // Add the JSON formatting layer
        .with(otel_layer) // Add the OpenTelemetry layer to link tracing and OTel contexts
        .init(); // Set this subscriber as the global default
}
