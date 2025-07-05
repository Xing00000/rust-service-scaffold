use axum::{http::StatusCode, response::IntoResponse, Json};
use contracts::{AppError, DomainError};
use serde::Serialize;

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        Self(e)
    }
}

#[derive(Serialize)]
struct ErrBody<'a> {
    code: &'a str,
    message: String,
}

#[derive(Serialize)]
struct ErrResp<'a> {
    error: ErrBody<'a>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code) = match &self.0 {
            AppError::Domain(DomainError::ValidationError { .. }) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR")
            }
            AppError::Domain(DomainError::NotFound { .. }) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::Domain(DomainError::BusinessRule { .. }) => {
                (StatusCode::BAD_REQUEST, "BUSINESS_RULE_VIOLATION")
            }
            AppError::Domain(DomainError::InvalidOperation { .. }) => {
                (StatusCode::BAD_REQUEST, "INVALID_OPERATION")
            }
            AppError::Infrastructure(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INFRASTRUCTURE_ERROR")
            }
            AppError::Application(_) => (StatusCode::INTERNAL_SERVER_ERROR, "APPLICATION_ERROR"),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
        };

        let body = ErrResp {
            error: ErrBody {
                code,
                message: self.0.to_string(),
            },
        };

        (status, Json(body)).into_response()
    }
}
