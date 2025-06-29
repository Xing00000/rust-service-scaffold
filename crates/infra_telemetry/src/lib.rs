pub mod config;
pub mod error;
pub mod metrics;
pub mod metrics_middleware;

pub mod telemetry; // 👈 新增

// 便捷 re‑export，供 app / presentation 使用
pub use metrics_middleware::axum_metrics_middleware as metrics_layer;
