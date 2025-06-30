use async_trait::async_trait;

use uuid::Uuid;

use domain::error::DomainError;
use domain::user::User;

//=== Port：UserRepository ===//
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait UserRepository: Send + Sync {
    async fn find(&self, id: &Uuid) -> Result<User, DomainError>;
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    async fn shutdown(&self); // Added for graceful shutdown
}
