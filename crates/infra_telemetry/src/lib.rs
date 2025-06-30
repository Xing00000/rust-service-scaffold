pub mod config;
pub mod error;
pub mod metrics;
// metrics_middleware was moved to pres_web_axum

pub mod telemetry; // 👈 新增

// 便捷 re‑export，供 app / presentation 使用
// pub use metrics_middleware::axum_metrics_middleware as metrics_layer; // This is now in pres_web_axum
