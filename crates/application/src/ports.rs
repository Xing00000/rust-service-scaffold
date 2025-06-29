use async_trait::async_trait;

use thiserror::Error;
use uuid::Uuid;

//=== Domain DTO 範例 ===//
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}

//=== 通用 Repository 錯誤 ===//
#[derive(Debug, Error)]
pub enum RepoError {
    #[error("entity not found")]
    NotFound,
    #[error("duplicate key")]
    Duplicate,
    #[error("unexpected repo error: {0}")]
    Unexpected(String),
}

//=== Port：UserRepository ===//
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find(&self, id: &Uuid) -> Result<User, RepoError>;
    async fn save(&self, user: &User) -> Result<(), RepoError>;
}
#[cfg(test)]
pub mod mock {
    use super::*;
    use mockall::automock;

    #[automock]
    #[async_trait::async_trait]
    pub trait UserRepository: Send + Sync {
        async fn find(&self, id: &uuid::Uuid) -> Result<User, RepoError>;
        async fn save(&self, user: &User) -> Result<(), RepoError>;
    }
}
