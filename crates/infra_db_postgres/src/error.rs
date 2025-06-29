use application::ports::RepoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
}

impl From<DbError> for RepoError {
    fn from(e: DbError) -> Self {
        match &e {
            DbError::Sqlx(sqlx::Error::RowNotFound) => RepoError::NotFound,
            DbError::Sqlx(err) if err.to_string().contains("duplicate key") => RepoError::Duplicate,
            _ => RepoError::Unexpected(e.to_string()),
        }
    }
}
