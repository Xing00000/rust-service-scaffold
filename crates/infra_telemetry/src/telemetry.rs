// src/infrastructure/telemetry.rs

use crate::config::TelemetryConfig;
use crate::error::TelemetryError;

use std::panic::PanicHookInfo;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// 初始化基本的 tracing subscriber
fn init_subscriber(config: &TelemetryConfig) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.log_level.clone()));

    let formatter = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true);

    Registry::default().with(env_filter).with(formatter).init();

    info!("Logging system initialized.");
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
pub fn init_telemetry(config: &TelemetryConfig) -> Result<prometheus::Registry, TelemetryError> {
    // 創建 Prometheus registry
    let registry = prometheus::Registry::new();
    info!("Metrics system (Prometheus registry) initialized.");

    // 初始化日誌系統
    init_subscriber(config);

    info!("Telemetry initialized successfully.");
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_telemetry_success() {
        let config = TelemetryConfig {
            log_level: "info".to_string(),
            otel_service_name: "test-service".to_string(),
            otel_exporter_otlp_endpoint: "http://localhost:4317".to_string(),
            prometheus_path: "/metrics".to_string(),
        };

        let result = init_telemetry(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_panic_hook_function_exists() {
        // Test that panic_hook function can be called without panicking
        // We can't easily create a PanicHookInfo in tests, so just verify the function exists
        let _fn_ptr = panic_hook as fn(&PanicHookInfo);
    }
}
