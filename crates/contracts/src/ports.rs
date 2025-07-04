use async_trait::async_trait;
use std::sync::Arc;

// Re-export domain types
pub use domain::{error::DomainError, user::User};
pub use uuid::Uuid;

//=== Repository Ports ===//
#[async_trait]
#[cfg_attr(any(test, feature = "testing"), mockall::automock)]
pub trait UserRepository: Send + Sync {
    async fn find(&self, id: &Uuid) -> Result<User, DomainError>;
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    async fn shutdown(&self);
}

//=== Observability Ports ===//
#[async_trait]
#[cfg_attr(any(test, feature = "testing"), mockall::automock)]
pub trait ObservabilityPort: Send + Sync {
    async fn on_request_start(&self, method: &str, path: &str);
    async fn on_request_end(&self, method: &str, path: &str, status: u16, latency: f64);
}

//=== Configuration Ports ===//
pub trait ConfigProvider: Send + Sync {
    type Config;
    fn get_config(&self) -> &Self::Config;
}

//=== Registry Ports ===//
pub trait MetricsRegistry: Send + Sync {
    fn registry(&self) -> &prometheus::Registry;
}

//=== Type Aliases ===//
pub type DynUserRepo = Arc<dyn UserRepository>;
pub type DynObservability = Arc<dyn ObservabilityPort>;
pub type DynMetricsRegistry = Arc<dyn MetricsRegistry>;
