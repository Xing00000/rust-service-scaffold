use application::error::AppError;
use axum::{http::StatusCode, response::IntoResponse, Json};
use contracts::ports::DomainError;
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
            AppError::Core(core_err) => {
                // You might want to log core_err here or handle different core errors differently
                tracing::error!("Core error: {:?}", core_err);
                (StatusCode::INTERNAL_SERVER_ERROR, "CORE_ERROR")
            }
            AppError::Domain(domain_error) => match domain_error {
                DomainError::Validation(_) => {
                    (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR")
                }
                DomainError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND_ERROR"),
                DomainError::Duplicate(_) => (StatusCode::CONFLICT, "DUPLICATE_ENTRY_ERROR"),
                DomainError::Unexpected(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "UNEXPECTED_DOMAIN_ERROR")
                } // Add other DomainError variants if any
            },
            // AppError::Repo variants were removed as RepoError was merged into DomainError.
            // All data access errors should now come through AppError::Domain.
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
