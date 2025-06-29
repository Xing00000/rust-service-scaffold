use application::{error::AppError, ports::RepoError};
use axum::{http::StatusCode, response::IntoResponse, Json};
use domain::error::DomainError;
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
            AppError::Core(_) => (StatusCode::BAD_REQUEST, "CORE"),
            AppError::Domain(domain_error) => match domain_error {
                DomainError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION"),
                DomainError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            },
            AppError::Repo(RepoError::NotFound) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::Repo(_) => (StatusCode::BAD_GATEWAY, "REPO"),
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
