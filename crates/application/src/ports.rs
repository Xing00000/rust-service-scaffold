use async_trait::async_trait;

use uuid::Uuid;

use domain::user::User;
use domain::error::DomainError;

//=== Port：UserRepository ===//
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find(&self, id: &Uuid) -> Result<User, DomainError>;
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    async fn shutdown(&self); // Added for graceful shutdown
}
#[cfg(test)]
pub mod mock {
    use super::*;
    use mockall::automock;
    use domain::error::DomainError; // Added for mock

    #[automock]
    #[async_trait::async_trait]
    pub trait UserRepository: Send + Sync {
        async fn find(&self, id: &uuid::Uuid) -> Result<User, DomainError>;
        async fn save(&self, user: &User) -> Result<(), DomainError>;
        async fn shutdown(&self); // Added for graceful shutdown
    }
}
