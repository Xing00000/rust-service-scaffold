use async_trait::async_trait;
use uuid::Uuid;

use crate::{error::DomainError, user::User};

/// 用戶儲存庫端口 - 屬於領域層
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find(&self, id: &Uuid) -> Result<User, DomainError>;
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    async fn shutdown(&self);
}

/// 可觀測性端口 - 屬於領域層
#[async_trait]
pub trait ObservabilityPort: Send + Sync {
    async fn on_request_start(&self, method: &str, path: &str);
    async fn on_request_end(&self, method: &str, path: &str, status: u16, latency: f64);
}