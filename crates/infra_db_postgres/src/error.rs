use contracts::{DomainError, InfraError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl From<DbError> for InfraError {
    fn from(e: DbError) -> Self {
        match e {
            DbError::Sqlx(err) => InfraError::Database(err.to_string()),
        }
    }
}

impl From<DbError> for DomainError {
    fn from(e: DbError) -> Self {
        match e {
            DbError::Sqlx(sqlx::Error::RowNotFound) => {
                DomainError::NotFound("Entity not found".to_string())
            }
            DbError::Sqlx(sqlx_err) => {
                if let Some(db_err) = sqlx_err.as_database_error() {
                    if db_err.is_unique_violation() {
                        return DomainError::InvalidOperation("Duplicate entry".to_string());
                    }
                }
                DomainError::InvalidOperation("Database operation failed".to_string())
            }
        }
    }
}
