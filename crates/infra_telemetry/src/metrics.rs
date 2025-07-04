use async_trait::async_trait;
use opentelemetry::{
    global,
    metrics::{Counter, Histogram},
    KeyValue,
};

#[derive(Clone)]
pub struct Metrics {
    http_requests_total: Counter<u64>,
    http_requests_duration_seconds: Histogram<f64>,
    http_requests_in_flight: opentelemetry::metrics::UpDownCounter<i64>,
}

impl Metrics {
    pub fn new() -> Self {
        let meter = global::meter("my-service.metrics");
        Self {
            http_requests_total: meter
                .u64_counter("http_requests")
                .with_description("Total HTTP requests")
                .build(),
            http_requests_duration_seconds: meter
                .f64_histogram("http_requests_duration_seconds")
                .with_description("HTTP request latency in seconds")
                .build(),
            http_requests_in_flight: meter
                .i64_up_down_counter("http_requests_in_flight")
                .with_description("Number of in-flight HTTP requests")
                .build(),
        }
    }

    pub fn on_request_start(&self, method: &str, path: &str) {
        let labels = [
            KeyValue::new("method", method.to_owned()),
            KeyValue::new("path", path.to_owned()),
        ];
        self.http_requests_in_flight.add(1, &labels);
    }

    pub fn on_request_end(&self, method: &str, path: &str, status: u16, latency: f64) {
        let labels = [
            KeyValue::new("method", method.to_owned()),
            KeyValue::new("path", path.to_owned()),
            KeyValue::new("status", status.to_string()),
        ];
        self.http_requests_in_flight.add(-1, &labels);
        self.http_requests_total.add(1, &labels);
        self.http_requests_duration_seconds.record(latency, &labels);
    }
}
impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl contracts::ports::ObservabilityPort for Metrics {
    async fn on_request_start(&self, method: &str, path: &str) {
        // Call the existing synchronous method.
        // In a real-world scenario, if the underlying metrics library supported async,
        // we might make this method async. For now, we wrap the sync call.
        Metrics::on_request_start(self, method, path);
    }

    async fn on_request_end(&self, method: &str, path: &str, status: u16, latency: f64) {
        // Call the existing synchronous method.
        Metrics::on_request_end(self, method, path, status, latency);
    }
}
