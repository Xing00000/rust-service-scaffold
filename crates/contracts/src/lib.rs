pub mod error;
pub use error::CoreError;

/// 全域設定介面：由呼叫端決定是否使用 `Arc`／`Rc`。
pub trait HasConfig {
    type Cfg;
    fn config(&self) -> &Self::Cfg;
}

/// Prometheus Registry 提供者
pub trait HasRegistry {
    fn registry(&self) -> &prometheus::Registry;
}

use application::ports::UserRepository;
use std::sync::Arc;

/// 可報告自身服務埠號
pub trait HasPort {
    fn port(&self) -> u16;
}

/// UserRepository 提供者
pub trait HasUserRepository {
    fn user_repo(&self) -> Arc<dyn UserRepository>;
}
