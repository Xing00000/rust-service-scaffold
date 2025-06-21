// src/infrastructure/telemetry.rs
// 負責初始化日誌和 OpenTelemetry 追踪。

use std::panic::PanicHookInfo;

use crate::config::Config;
use crate::infrastructure::error::InfrastructureError;
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// 根據配置初始化 OTLP 追踪器
pub fn init_tracer(config: &Config) -> Result<sdktrace::Tracer, TraceError> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.otel_exporter_otlp_endpoint);

    let resource = Resource::new(vec![opentelemetry::KeyValue::new(
        SERVICE_NAME,
        config.otel_service_name.clone(),
    )]);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(sdktrace::Config::default().with_resource(resource))
        .install_batch(runtime::Tokio)?;

    Ok(tracer)
}

/// 初始化 `tracing` subscriber，可選擇性地集成 OpenTelemetry layer。
pub fn init_subscriber(config: &Config, tracer: Option<sdktrace::Tracer>) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.log_level.clone()));

    let formatter = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true);

    let subscriber = Registry::default().with(env_filter).with(formatter);

    if let Some(tracer) = tracer {
        let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);
        subscriber.with(otel_layer).init();
        info!("OpenTelemetry layer initialized.");
    } else {
        subscriber.init();
        info!("OpenTelemetry layer NOT initialized (no tracer provided).");
    }
}

/// 全局 Panic Hook，用於捕獲和記錄 panic。
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
pub fn init_telemetry(config: &Config) -> Result<(), InfrastructureError> {
    let otel_tracer =
        init_tracer(config).map_err(|e| InfrastructureError::TelemetryInit(e.to_string()))?;
    init_subscriber(config, Some(otel_tracer));
    Ok(())
}
