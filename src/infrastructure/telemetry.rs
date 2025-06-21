// src/infrastructure/telemetry.rs

use crate::config::Config;
use crate::infrastructure::error::InfrastructureError;

use opentelemetry::{
    global,
    trace::{TraceError, TracerProvider},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SdkTracerProvider, Resource};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use std::panic::PanicHookInfo;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// 使用 Pipeline Builder 初始化 OTLP 追踪器。
fn init_tracer_provider(
    config: &Config,
    resource: Resource,
) -> Result<SdkTracerProvider, TraceError> {
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(&config.otel_exporter_otlp_endpoint)
        .build()?;

    Ok(opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(resource)
        .with_simple_exporter(otlp_exporter)
        .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
        .build())
}

/// 初始化 `tracing` subscriber，並集成 OpenTelemetry layer。
fn init_subscriber(config: &Config, provider: SdkTracerProvider) {
    let tracer = provider.tracer("tracing-opentelemetry");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    global::set_tracer_provider(provider);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.log_level.clone()));

    let formatter = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true);

    Registry::default()
        .with(env_filter)
        .with(formatter)
        .with(otel_layer)
        .init();

    info!("OpenTelemetry layer initialized.");
}

/// 全局 Panic Hook
pub fn panic_hook(panic_info: &PanicHookInfo) {
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

    let location = panic_info.location().map(|loc| loc.to_string());
    let backtrace = std::backtrace::Backtrace::capture();

    tracing::error!(
        target: "panic",
        payload = payload,
        location = ?location,
        backtrace = ?backtrace,
        "A panic occurred"
    );
}

/// 完整的遙測初始化流程
pub fn init_telemetry(config: &Config) -> Result<prometheus::Registry, InfrastructureError> {
    let resource = Resource::builder()
        .with_attributes(vec![KeyValue::new(
            SERVICE_NAME,
            config.otel_service_name.clone(),
        )])
        .build();

    let registry = prometheus::Registry::new();

    // --- 初始化指標系統 (使用 opentelemetry-prometheus) ---
    // 1. 創建 Prometheus 導出器
    let prometheus_exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone()) // <- 使用新的 Registry
        .build()
        .map_err(|e| InfrastructureError::MetricsInit(e.to_string()))?;

    // 初始化指標系統
    let meter_provider = SdkMeterProvider::builder()
        .with_resource(resource.clone()) // resource 可以被克隆
        .with_reader(prometheus_exporter)
        .build();

    global::set_meter_provider(meter_provider);

    info!("Metrics system (Prometheus exporter) initialized.");

    // 初始化追踪系統
    let tracer_provider = init_tracer_provider(config, resource)
        .map_err(|e| InfrastructureError::TelemetryInit(e.to_string()))?;

    // 初始化日誌系統並與追踪集成
    init_subscriber(config, tracer_provider);

    info!("Telemetry initialized successfully.");
    Ok(registry)
}
