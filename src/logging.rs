use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_subscriber::fmt::Layer as FmtLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use opentelemetry_sdk::trace::Tracer as SdkTracer; // Import and alias SdkTracer
use tracing::info; // For logging the message when OTel is not initialized

pub fn init_subscriber(tracer: Option<SdkTracer>) { // Updated signature
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let formatter = FmtLayer::default()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .with_thread_ids(true)
        .with_thread_names(true);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(formatter);

    if let Some(tracer) = tracer {
        let otel_layer = OpenTelemetryLayer::new(tracer);
        subscriber.with(otel_layer).init();
        info!("OpenTelemetry layer initialized.");
    } else {
        subscriber.init();
        info!("OpenTelemetry layer NOT initialized (no tracer provided).");
    }
}
