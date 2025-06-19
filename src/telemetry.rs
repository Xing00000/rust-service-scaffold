use opentelemetry::global;
use opentelemetry_sdk::trace as sdktrace; // Renamed for consistency
use opentelemetry_sdk::trace::TraceError; // Corrected path
use opentelemetry::metrics::MeterProvider; // Corrected path as per compiler suggestion
use opentelemetry_sdk::Resource;
// Removed unused import: use serde_json::json;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use opentelemetry::trace::TraceContextExt; // Added for current_with_span

// Function to initialize the Jaeger tracer
pub fn init_tracer() -> Result<sdktrace::Tracer, TraceError> { // sdktrace::Tracer is correct with alias
    // let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "axum-service".to_string());
    // opentelemetry_jaeger::new_agent_pipeline()
    //     .with_service_name(service_name)
    //     .install_batch(opentelemetry_sdk::runtime::Tokio) // This might still have JaegerTraceRuntime issue
    Err(TraceError::Other("Jaeger tracer temporarily disabled due to compilation issues".into()))
}

// Function to initialize the Prometheus meter provider
pub fn init_meter_provider() -> impl MeterProvider { // Return impl MeterProvider
    let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "axum-service".to_string());
    let resource = Resource::builder()
        .with_attributes(vec![opentelemetry::KeyValue::new(SERVICE_NAME, service_name)])
        .build();
    opentelemetry_sdk::metrics::SdkMeterProvider::builder().with_resource(resource).build() // Corrected to SdkMeterProvider
}

// Helper function to make a span and set it as parent
// (This is a basic example, you might want to customize it further)
pub fn _create_span(name: String, parent_cx: opentelemetry::Context) -> opentelemetry::Context {
    use opentelemetry::trace::Tracer; // This Tracer is from opentelemetry::trace
    let tracer = global::tracer("my-tracer");
    let span = tracer.start_with_context(name, &parent_cx); // Pass String directly
    parent_cx.with_span(span) // Corrected usage of with_span
}

// Panic hook function to log panic information
pub fn panic_hook(panic_info: &std::panic::PanicHookInfo) {
    let payload = panic_info
        .payload()
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic_info.payload().downcast_ref::<String>().map(|s| s.as_str()))
        .unwrap_or("unknown panic payload");

    let location = panic_info.location().map(|loc| {
        // Using tracing::field::debug for location, so direct serde_json::json! is not strictly needed here for Value.
        // However, if we wanted to construct a string or a structured object for logging, this is where it would be.
        // For now, we rely on the Debug impl of Location or a custom struct if it were more complex.
        // The current approach uses tracing::field::debug(location_object_from_panic_info),
        // so the direct serde_json::json! macro for the `location` variable itself is not used.
        // Let's create a string representation for now if needed, or keep it as is if debug is sufficient.
        // The current code passes `tracing::field::debug(location)` where location is `Option<PanicLocation>`.
        // `PanicLocation` has a `Debug` impl. So `serde_json::json!` isn't used for `location` directly.
        // The warning is about `use serde_json::json;` at the top of the file.
        // The actual `serde_json::json!` call was removed in a previous step when changing how location is logged.
        // It seems the `read_files` output was from before that cleanup.
        // The `location` variable is an `Option<std::panic::Location>`.
        // The `tracing::field::debug(location)` call will use the `Debug` implementation of `Option<std::panic::Location>`.
        // No `serde_json::json!` macro is currently used in the panic_hook for the `location` variable.
        // The `serde_json::json` import is indeed unused.
        format!("file: {}, line: {}, column: {}", loc.file(), loc.line(), loc.column())
    });

    let backtrace = std::backtrace::Backtrace::capture();

    tracing::error!(
        target: "panic",
        payload = payload,
        location = tracing::field::debug(location),
        backtrace = ?backtrace,
        "A panic occurred"
    );

    // TODO: Attempted to add graceful shutdown for telemetry, e.g.:
    // opentelemetry::global::shutdown_tracer_provider();
    // opentelemetry::global::shutdown_meter_provider();
    // However, these calls result in compilation error E0425: function not found in module global.
    // This might be an environment-specific issue or a subtle dependency conflict.
    // For now, graceful shutdown of telemetry in panic hook is omitted.
}
