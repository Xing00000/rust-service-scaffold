use thiserror::Error;
use domain::error::DomainError;

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

/// HTTP 狀態碼映射
impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::Domain(DomainError::NotFound(_)) => 404,
            AppError::Domain(DomainError::BusinessRule(_)) => 400,
            AppError::Domain(DomainError::InvalidOperation(_)) => 400,
            AppError::Domain(DomainError::Validation(_)) => 400,
            AppError::Validation(_) => 400,
            AppError::Infrastructure(_) => 500,
            AppError::Application(_) => 500,
        }
    }
}

// 向後兼容
pub type CoreError = AppError;
