// src/error.rs
use crate::domain::error::DomainError;
use crate::infrastructure::error::InfrastructureError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Internal Server Error")]
    InternalServerError,

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Infrastructure(#[from] InfrastructureError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            // ✅ 修正: 借用 self
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error occurred".to_string(),
            ),
            // ✅ 修正: 使用 ref 借用 msg，避免移動
            AppError::Domain(DomainError::ValidationError(ref msg)) => {
                (StatusCode::UNPROCESSABLE_ENTITY, msg.clone())
            }
            AppError::Infrastructure(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error occurred".to_string(),
            ),
        };

        // ✅ 修正: 現在 self 沒有被移動，可以安全地借用
        tracing::error!(error = ?self, "Request failed");

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
