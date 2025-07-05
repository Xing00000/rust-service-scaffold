use async_trait::async_trait;
use std::sync::Arc;

// Re-export domain types and ports
pub use domain::{error::DomainError, user::User, UserId, UserRepository};
pub use uuid::Uuid;

//=== Application Layer Ports ===//
/// 可觀測性端口 - 屬於應用層橫切關注
#[async_trait]
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

// Mock implementations for testing
#[cfg(any(test, feature = "testing"))]
pub use mockall::mock;

#[cfg(any(test, feature = "testing"))]
mock! {
    pub ObservabilityPort {}
    
    #[async_trait]
    impl ObservabilityPort for ObservabilityPort {
        async fn on_request_start(&self, method: &str, path: &str);
        async fn on_request_end(&self, method: &str, path: &str, status: u16, latency: f64);
    }
}
