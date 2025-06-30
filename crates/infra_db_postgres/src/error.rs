use domain::error::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    // Potentially other DB-specific errors can be added here
}

impl From<DbError> for DomainError {
    fn from(e: DbError) -> Self {
        match e {
            DbError::Sqlx(sqlx::Error::RowNotFound) => {
                DomainError::NotFound("Entity not found in database".to_string())
            }
            DbError::Sqlx(sqlx_err) => {
                if let Some(db_err) = sqlx_err.as_database_error() {
                    if db_err.is_unique_violation() {
                        return DomainError::Duplicate(format!("Database unique constraint violation: {}", db_err));
                    }
                }
                DomainError::Unexpected(format!("Database operation failed: {}", sqlx_err))
            }
        }
    }
}
