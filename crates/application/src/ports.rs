use async_trait::async_trait;
use std::sync::Arc; // Added to resolve E0412 for DynObs

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

//=== Port：ObservabilityPort ===//
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait ObservabilityPort: Send + Sync {
    async fn on_request_start(&self, method: &str, path: &str);
    async fn on_request_end(&self, method: &str, path: &str, status: u16, latency: f64);
    // TODO: Add methods for recording metrics, e.g.:
    // async fn record_custom_metric(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}

pub type DynObs = Arc<dyn ObservabilityPort + Send + Sync>;
