use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
// Ensure serde_json::json is available if you prefer direct construction,
// but using structs is cleaner.
// use serde_json::json;
use std::error::Error; // Required for err.source()
use thiserror::Error;
use tracing::error; // Import tracing::error

// Define the structure for the JSON error response
#[derive(Serialize)]
struct ErrorDetails {
    code: String,
    message: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorDetails,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("An internal server error occurred.")]
    InternalServerError,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Failed to (de)serialize JSON: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    // Placeholder for future Reqwest errors
    // #[error("External API request failed: {0}")]
    // ReqwestError(#[from] reqwest::Error),

    // Placeholder for future SQLx errors
    // #[error("Database error: {0}")]
    // SqlxError(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, error_message) = match &self {
            AppError::InternalServerError => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR".to_string(),
                    "An internal server error occurred.".to_string(),
                )
            }
            AppError::BadRequest(reason) => {
                (
                    StatusCode::BAD_REQUEST,
                    "BAD_REQUEST".to_string(),
                    reason.clone(),
                )
            }
            AppError::SerdeJsonError(ref _err) => { // _err is not directly used in client message
                (
                    StatusCode::UNPROCESSABLE_ENTITY, // Or BAD_REQUEST
                    "INVALID_JSON".to_string(),
                    "The request body contained invalid JSON.".to_string(),
                )
            }
        };

        // Log the error with more details (including source for SerdeJsonError)
        // Ensure that the Display impl of AppError (from thiserror) is suitable for logging.
        // For SerdeJsonError, we also want to log the source.
        match &self {
            AppError::SerdeJsonError(ref err) => {
                if let Some(source) = err.source() {
                    error!(error.message = %self, error.source = %source, "Handling AppError");
                } else {
                    error!(error.message = %self, "Handling AppError");
                }
            }
            _ => {
                error!(error.message = %self, "Handling AppError");
            }
        }

        let body = Json(ErrorResponse {
            error: ErrorDetails {
                code: error_code,
                message: error_message,
            },
        });

        (status, body).into_response()
    }
}
