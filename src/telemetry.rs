use opentelemetry::global;
use opentelemetry::metrics::MeterProvider;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig; // Added for OTLP exporter
use opentelemetry_sdk::trace as sdktrace;
use opentelemetry_sdk::trace::Config as SdkTraceConfig; // Aliased to avoid conflict if opentelemetry::trace::Config is used
use opentelemetry_sdk::{runtime, Resource}; // Added runtime
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use std::env; // For reading environment variables

// Function to initialize the OTLP tracer
pub fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    let otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let service_name =
        env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "axum-logging-service".to_string());

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(otlp_endpoint);

    let resource = Resource::new(vec![opentelemetry::KeyValue::new(
        SERVICE_NAME,
        service_name,
    )]);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(SdkTraceConfig::default().with_resource(resource))
        .install_batch(runtime::Tokio)?;

    Ok(tracer)
}

// Function to initialize the Prometheus meter provider
pub fn init_meter_provider() -> impl MeterProvider {
    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "axum-service".to_string());
    let resource = Resource::new(vec![opentelemetry::KeyValue::new(
        SERVICE_NAME,
        service_name,
    )]);
    opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_resource(resource)
        .build()
}

// Helper function to make a span and set it as parent
pub fn _create_span(name: String, parent_cx: opentelemetry::Context) -> opentelemetry::Context {
    use opentelemetry::trace::Tracer;
    let tracer = global::tracer("my-tracer");
    let span = tracer.start_with_context(name, &parent_cx);
    parent_cx.with_span(span)
}

// Panic hook function to log panic information
pub fn panic_hook(panic_info: &std::panic::PanicHookInfo) {
    tracing::error!(target: "panic_hook_test_sentinel", "PANIC_HOOK_ACTIVATED_SENTINEL_MESSAGE");

    let payload = panic_info
        .payload()
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| {
            panic_info
                .payload()
                .downcast_ref::<String>()
                .map(|s| s.as_str())
        })
        .unwrap_or("unknown panic payload");

    let location = panic_info.location().map(|loc| {
        format!(
            "file: {}, line: {}, column: {}",
            loc.file(),
            loc.line(),
            loc.column()
        )
    });

    let backtrace = std::backtrace::Backtrace::capture();

    tracing::error!(
        target: "panic",
        payload = payload,
        location = tracing::field::debug(location),
        backtrace = ?backtrace,
        "A panic occurred"
    );
}
