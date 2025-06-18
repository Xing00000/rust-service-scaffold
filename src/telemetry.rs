use opentelemetry::{global, sdk::trace as sdktrace, trace::TraceError};
use opentelemetry_sdk::metrics::MeterProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;

// Function to initialize the Jaeger tracer
pub fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "axum-service".to_string());
    opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(service_name)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

// Function to initialize the Prometheus meter provider
pub fn init_meter_provider() -> MeterProvider {
    let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "axum-service".to_string());
    let resource = Resource::new(vec![opentelemetry::KeyValue::new(SERVICE_NAME, service_name)]);
    MeterProvider::builder().with_resource(resource).build()
}

// Helper function to make a span and set it as parent
// (This is a basic example, you might want to customize it further)
pub fn _create_span(name: &str, parent_cx: opentelemetry::Context) -> opentelemetry::Context {
    use opentelemetry::trace::Tracer;
    let tracer = global::tracer("my-tracer");
    let span = tracer.start_with_context(name, &parent_cx);
    opentelemetry::Context::current_with_span(span)
}
