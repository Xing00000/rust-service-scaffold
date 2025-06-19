use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_subscriber::fmt::Layer as FmtLayer; // Removed unused 'self'
use tracing_opentelemetry::OpenTelemetryLayer;
// Remove opentelemetry::global, as we'll use a specific tracer
// use opentelemetry::global;
use crate::telemetry; // Import the telemetry module to access init_tracer

pub fn init_subscriber() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info")); // Default to info level, can be overridden by RUST_LOG

    // Create a formatting layer (e.g., JSON)
    let formatter = FmtLayer::default()
        .json() // Output logs in JSON format
        .with_current_span(true) // Include current span info
        .with_span_list(true) // Include span list (shows parent spans)
        .with_timer(tracing_subscriber::fmt::time::SystemTime) // Use SystemTime for timestamps
        .with_thread_ids(true) // Include thread IDs
        .with_thread_names(true); // Include thread names

    // Initialize our tracer
    match telemetry::init_tracer() {
        Ok(tracer) => {
            let otel_layer = OpenTelemetryLayer::new(tracer);
            Registry::default()
                .with(env_filter)
                .with(formatter) // Add the JSON formatting layer
                .with(otel_layer) // Add the OpenTelemetry layer to link tracing and OTel contexts
                .init(); // Set this subscriber as the global default
        },
        Err(e) => {
            eprintln!("Failed to initialize OpenTelemetry tracer: {:?}. Proceeding without OpenTelemetry layer.", e);
            Registry::default()
                .with(env_filter)
                .with(formatter) // Add the JSON formatting layer
                .init(); // Set this subscriber as the global default without OTel
        }
    };
}
