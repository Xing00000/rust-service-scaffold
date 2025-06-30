// infra_telemetry/src/config.rs

//! Telemetry (tracing, metrics) 相關設定
//! 此檔僅定義 Telemetry Adapter 所需的設定 struct 與介面。

use serde::Deserialize;

/// Telemetry 相關配置。
/// 注意：此 Config 僅限 Telemetry Adapter 需要的設定，與全域 Config 解耦。
#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    /// 服務名稱（用於 tracing、Prometheus 標籤）
    pub otel_service_name: String,

    /// OTLP Endpoint (tracing)
    pub otel_exporter_otlp_endpoint: String,

    /// Prometheus metrics HTTP 路徑
    #[serde(default = "default_prometheus_path")]
    pub prometheus_path: String,

    /// 日誌等級（trace/debug/info/warn/error）
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_prometheus_path() -> String {
    "/metrics".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

impl TelemetryConfig {
    /// 可選：從 env 或 toml 讀取 Telemetry 設定（非強制）
    /// Marked for test use only to enforce DI for main application flow.
    #[cfg(test)]
    pub fn from_env() -> Self {
        use std::env;
        TelemetryConfig {
            otel_service_name: env::var("TELEMETRY_SERVICE_NAME").unwrap_or_else(|_| "app".into()),
            otel_exporter_otlp_endpoint: env::var("TELEMETRY_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".into()),
            prometheus_path: env::var("TELEMETRY_PROMETHEUS_PATH")
                .unwrap_or_else(|_| default_prometheus_path()),
            log_level: env::var("TELEMETRY_LOG_LEVEL").unwrap_or_else(|_| default_log_level()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        // Renamed for clarity as from_env is now test-only
        let cfg = TelemetryConfig {
            otel_service_name: "svc".into(),
            otel_exporter_otlp_endpoint: "http://otlp".into(),
            prometheus_path: default_prometheus_path(),
            log_level: default_log_level(),
        };
        assert_eq!(cfg.prometheus_path, "/metrics");
        assert_eq!(cfg.log_level, "info");
    }

    #[test]
    #[cfg(test)] // This test specifically uses from_env
    fn test_from_env_defaults() {
        // 清理環境變數
        std::env::remove_var("TELEMETRY_SERVICE_NAME");
        std::env::remove_var("TELEMETRY_OTLP_ENDPOINT");
        std::env::remove_var("TELEMETRY_PROMETHEUS_PATH");
        std::env::remove_var("TELEMETRY_LOG_LEVEL");
        let cfg = TelemetryConfig::from_env();
        assert_eq!(cfg.otel_service_name, "app");
        assert_eq!(cfg.otel_exporter_otlp_endpoint, "http://localhost:4317");
        assert_eq!(cfg.prometheus_path, "/metrics");
        assert_eq!(cfg.log_level, "info");
    }
}
