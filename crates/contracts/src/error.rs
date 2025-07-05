use domain::error::DomainError;
use thiserror::Error;

/// 統一的應用錯誤類型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] InfraError),

    #[error("Application error: {0}")]
    Application(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Error)]
pub enum InfraError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

// HTTP 狀態碼映射已移除 - 由 presentation 層負責

// 向後兼容
pub type CoreError = AppError;
