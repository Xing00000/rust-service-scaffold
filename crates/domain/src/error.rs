use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("Duplicate entry: {0}")]
    Duplicate(String),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}
