// src/infrastructure/web/metrics.rs

use axum::{
    extract::{MatchedPath, Request},
    response::IntoResponse,
};

use once_cell::sync::Lazy;

use opentelemetry::{
    global,
    metrics::{Counter, Histogram},
    KeyValue,
};
use std::time::Instant;

// 使用 once_cell::sync::Lazy 來確保我們的指標只被初始化一次。
// 這是建立全域單例的常見 Rust 模式。
static METRICS: Lazy<Metrics> = Lazy::new(|| {
    let meter = global::meter("my-rust-service.web");
    Metrics {
        http_requests_total: meter
            .u64_counter("http_requests")
            .with_description("Total number of HTTP requests made.")
            .build(),
        http_requests_duration_seconds: meter
            .f64_histogram("http_requests_duration_seconds")
            .with_description("The HTTP request latencies in seconds.")
            .build(),
        http_requests_in_flight: meter
            .i64_up_down_counter("http_requests_in_flight")
            .with_description("The number of in-flight HTTP requests.")
            .build(),
    }
});

#[derive(Clone)]
pub struct Metrics {
    pub http_requests_total: Counter<u64>,
    pub http_requests_duration_seconds: Histogram<f64>,
    pub http_requests_in_flight: opentelemetry::metrics::UpDownCounter<i64>,
}

// Axum middleware 函式
pub async fn track_metrics(req: Request, next: axum::middleware::Next) -> impl IntoResponse {
    // 獲取匹配的路由路徑，例如 "/users/:id"，而不是實際的 "/users/123"
    // 這對於避免指標標籤爆炸性增長至關重要。
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };

    let method = req.method().clone();
    let start = Instant::now();

    // 記錄進行中的請求
    let common_labels = [
        KeyValue::new("method", method.to_string()),
        KeyValue::new("path", path.clone()),
    ];
    METRICS.http_requests_in_flight.add(1, &common_labels);

    // 執行下一個 middleware 或 handler
    let response = next.run(req).await;

    // 處理完成，減少進行中的請求計數
    METRICS.http_requests_in_flight.add(-1, &common_labels);

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // 最終的指標標籤，包含狀態碼
    let final_labels = [
        KeyValue::new("method", method.to_string()),
        KeyValue::new("path", path),
        KeyValue::new("status", status),
    ];

    // 記錄請求總數和延遲
    METRICS.http_requests_total.add(1, &final_labels);
    METRICS
        .http_requests_duration_seconds
        .record(latency, &final_labels);

    response
}
