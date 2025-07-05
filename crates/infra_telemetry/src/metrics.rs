use crate::config::TelemetryConfig;
use async_trait::async_trait;
use opentelemetry::{
    global,
    metrics::{Counter, Histogram},
    KeyValue,
};

const HTTP_REQUESTS_TOTAL: &str = "http_requests_total";
const HTTP_REQUESTS_DURATION: &str = "http_requests_duration_seconds";
const HTTP_REQUESTS_IN_FLIGHT: &str = "http_requests_in_flight";

#[derive(Clone)]
pub struct Metrics {
    http_requests_total: Counter<u64>,
    http_requests_duration_seconds: Histogram<f64>,
    http_requests_in_flight: opentelemetry::metrics::UpDownCounter<i64>,
}

impl Metrics {
    pub fn new(config: &TelemetryConfig) -> Self {
        let service_name: &'static str =
            Box::leak(config.otel_service_name.clone().into_boxed_str());
        let meter = global::meter(service_name);
        Self {
            http_requests_total: meter
                .u64_counter(HTTP_REQUESTS_TOTAL)
                .with_description("Total HTTP requests")
                .build(),
            http_requests_duration_seconds: meter
                .f64_histogram(HTTP_REQUESTS_DURATION)
                .with_description("HTTP request latency in seconds")
                .build(),
            http_requests_in_flight: meter
                .i64_up_down_counter(HTTP_REQUESTS_IN_FLIGHT)
                .with_description("Number of in-flight HTTP requests")
                .build(),
        }
    }

    fn create_labels(method: &str, path: &str) -> Vec<KeyValue> {
        vec![
            KeyValue::new("method", method.to_owned()),
            KeyValue::new("path", path.to_owned()),
        ]
    }

    fn create_labels_with_status(method: &str, path: &str, status: u16) -> Vec<KeyValue> {
        vec![
            KeyValue::new("method", method.to_owned()),
            KeyValue::new("path", path.to_owned()),
            KeyValue::new("status", status.to_string()),
        ]
    }

    pub fn on_request_start(&self, method: &str, path: &str) {
        let labels = Self::create_labels(method, path);
        self.http_requests_in_flight.add(1, &labels);
    }

    pub fn on_request_end(&self, method: &str, path: &str, status: u16, latency: f64) {
        let base_labels = Self::create_labels(method, path);
        let status_labels = Self::create_labels_with_status(method, path, status);

        self.http_requests_in_flight.add(-1, &base_labels);
        self.http_requests_total.add(1, &status_labels);
        self.http_requests_duration_seconds
            .record(latency, &status_labels);
    }
}

impl Default for Metrics {
    fn default() -> Self {
        let default_config = TelemetryConfig {
            otel_service_name: "default-service".to_string(),
            otel_exporter_otlp_endpoint: "http://localhost:4317".to_string(),
            prometheus_path: "/metrics".to_string(),
            log_level: "info".to_string(),
        };
        Self::new(&default_config)
    }
}

#[async_trait]
impl contracts::ObservabilityPort for Metrics {
    async fn on_request_start(&self, method: &str, path: &str) {
        Metrics::on_request_start(self, method, path);
    }

    async fn on_request_end(&self, method: &str, path: &str, status: u16, latency: f64) {
        Metrics::on_request_end(self, method, path, status, latency);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let config = TelemetryConfig {
            otel_service_name: "test-service".to_string(),
            otel_exporter_otlp_endpoint: "http://localhost:4317".to_string(),
            prometheus_path: "/metrics".to_string(),
            log_level: "info".to_string(),
        };
        let metrics = Metrics::new(&config);
        // 測試指標對象創建成功 - 檢查結構體存在
        let _counter = &metrics.http_requests_total;
        let _histogram = &metrics.http_requests_duration_seconds;
        let _gauge = &metrics.http_requests_in_flight;
        // 測試通過表示指標已正確初始化
    }

    #[test]
    fn test_create_labels() {
        let labels = Metrics::create_labels("GET", "/api/users");
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].key.as_str(), "method");
        assert_eq!(labels[1].key.as_str(), "path");
    }

    #[test]
    fn test_create_labels_with_status() {
        let labels = Metrics::create_labels_with_status("POST", "/api/users", 201);
        assert_eq!(labels.len(), 3);
        assert_eq!(labels[2].key.as_str(), "status");
    }
}
