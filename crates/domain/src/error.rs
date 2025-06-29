use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Validation error: {0}")] // <--- 修正這裡
    Validation(String),
    #[error("Entity not found: {0}")]
    NotFound(String),
}
