use thiserror::Error;

/// 專屬 Repository 抽象錯誤
#[derive(Debug, Error)]
pub enum RepoError {
    #[error("entity not found")]
    NotFound,
    #[error("duplicate key")]
    Duplicate,
    #[error("unexpected repo error: {0}")]
    Unexpected(String),
}
