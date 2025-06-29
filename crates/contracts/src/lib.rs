pub mod error;
pub use error::CoreError;
use std::sync::Arc;

/// 這是 app-wide 共用設定的抽象；你可以用關聯型別或泛型化
pub trait HasConfig {
    type Cfg;
    fn config(&self) -> &Arc<Self::Cfg>;
}

/// 如果還有 Prometheus Registry 需求
pub trait HasRegistry {
    fn registry(&self) -> &Arc<prometheus::Registry>;
}

pub trait HasPort {
    fn port(&self) -> u16;
}
