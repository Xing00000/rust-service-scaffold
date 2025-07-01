//! Cross-layer error definition. No web / db / tokio deps.
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Internal server error")]
    Internal,
}
