// src/infrastructure/error.rs
// 定義與基礎設施相關的錯誤，如數據庫連接、外部 API 調用失敗等。

use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfrastructureError {
    #[error("Telemetry initialization failed: {0}")]
    TelemetryInit(String),
    #[error("Telemetry (Metrics) initialization failed: {0}")]
    MetricsInit(String),
    // #[error("Database connection failed")]
    // DatabaseConnection(#[from] sqlx::Error), // 示例，需要添加 sqlx 依賴
}
