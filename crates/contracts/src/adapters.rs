//=== Domain Port Adapters ===//

use async_trait::async_trait;
use domain::{error::DomainError, user::User, id::UserId, UserRepository as DomainUserRepository};

/// Domain UserRepository 的 async-trait 適配器
/// 將 Domain 的純 Future trait 適配為 async-trait，方便基礎設施層實現
#[async_trait]
pub trait UserRepositoryAdapter: Send + Sync {
    async fn find(&self, id: &UserId) -> Result<User, DomainError>;
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    async fn shutdown(&self);
}

/// 為任何實現 DomainUserRepository 的類型自動實現 UserRepositoryAdapter
#[async_trait]
impl<T> UserRepositoryAdapter for T
where
    T: DomainUserRepository + Send + Sync,
{
    async fn find(&self, id: &UserId) -> Result<User, DomainError> {
        DomainUserRepository::find(self, id).await
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        DomainUserRepository::save(self, user).await
    }

    async fn shutdown(&self) {
        DomainUserRepository::shutdown(self).await
    }
}

// Mock implementations for testing
#[cfg(any(test, feature = "testing"))]
pub use mockall::mock;

#[cfg(any(test, feature = "testing"))]
mock! {
    pub UserRepositoryAdapter {}
    
    #[async_trait]
    impl UserRepositoryAdapter for UserRepositoryAdapter {
        async fn find(&self, id: &UserId) -> Result<User, DomainError>;
        async fn save(&self, user: &User) -> Result<(), DomainError>;
        async fn shutdown(&self);
    }
}