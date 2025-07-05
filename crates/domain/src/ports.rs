use std::future::Future;
use std::pin::Pin;

use crate::{error::DomainError, id::UserId, user::User};

/// 用戶儲存庫端口 - 屬於領域層（純 Rust 實現）
pub trait UserRepository: Send + Sync {
    fn find(&self, id: &UserId) -> Pin<Box<dyn Future<Output = Result<User, DomainError>> + Send + '_>>;
    fn save(&self, user: &User) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + '_>>;
    fn shutdown(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}