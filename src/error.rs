// src/error.rs
//! 全域錯誤與結構化 HTTP 響應
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

/// ========= 領域層（Domain）錯誤 =========
pub use crate::domain::error::DomainError;
/// ========= 基礎設施（Infrastructure）錯誤 =========
pub use crate::infrastructure::error::InfrastructureError;

/// ========= 應用層統一錯誤 =========
#[derive(Debug, Error)]
pub enum AppError {
    // ... 保持不變
    #[error("{0}")]
    Domain(#[from] DomainError),
    #[error("{0}")]
    Infrastructure(#[from] InfrastructureError),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    #[error("Unexpected internal error")]
    Internal,
}

/// ========= 結構化錯誤載體 =========
#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: String,
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    error: ErrorBody<'a>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(error.details = ?self, "Request resulted in an error");

        // 將 enum variant 映射到 HTTP 狀態碼與錯誤碼字串
        let (status, code) = match &self {
            AppError::Domain(_) => (StatusCode::BAD_REQUEST, "DOMAIN_ERROR"),
            AppError::Infrastructure(_) => (StatusCode::BAD_GATEWAY, "INFRA_ERROR"),
            AppError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION"),
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            AppError::NotFound { .. } => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL"),
        };

        // ✅ 修正: 構建嵌套的 body 結構
        let body = ErrorResponse {
            error: ErrorBody {
                code,
                message: self.to_string(),
            },
        };

        (status, Json(body)).into_response()
    }
}
