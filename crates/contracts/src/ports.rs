use std::sync::Arc;

// Re-export domain types and ports
pub use domain::{error::DomainError, user::User, UserRepository, ObservabilityPort};
pub use uuid::Uuid;

// Mock implementations for testing
#[cfg(any(test, feature = "testing"))]
pub use mockall::mock;

#[cfg(any(test, feature = "testing"))]
mock! {
    pub UserRepository {}
    
    #[async_trait::async_trait]
    impl domain::UserRepository for UserRepository {
        async fn find(&self, id: &Uuid) -> Result<User, DomainError>;
        async fn save(&self, user: &User) -> Result<(), DomainError>;
        async fn shutdown(&self);
    }
}

#[cfg(any(test, feature = "testing"))]
mock! {
    pub ObservabilityPort {}
    
    #[async_trait::async_trait]
    impl domain::ObservabilityPort for ObservabilityPort {
        async fn on_request_start(&self, method: &str, path: &str);
        async fn on_request_end(&self, method: &str, path: &str, status: u16, latency: f64);
    }
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
pub type DynUserRepo = Arc<dyn domain::UserRepository>;
pub type DynObservability = Arc<dyn domain::ObservabilityPort>;
pub type DynMetricsRegistry = Arc<dyn MetricsRegistry>;
