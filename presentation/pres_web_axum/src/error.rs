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
        let status =
            StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let code = match &self.0 {
            AppError::Domain(DomainError::Validation(_)) => "VALIDATION_ERROR",
            AppError::Domain(DomainError::NotFound(_)) => "NOT_FOUND",
            AppError::Domain(DomainError::BusinessRule(_)) => "BUSINESS_RULE_VIOLATION",
            AppError::Domain(DomainError::InvalidOperation(_)) => "INVALID_OPERATION",
            AppError::Infrastructure(_) => "INFRASTRUCTURE_ERROR",
            AppError::Application(_) => "APPLICATION_ERROR",
            AppError::Validation(_) => "VALIDATION_ERROR",
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
