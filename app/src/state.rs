use application::ports::UserRepository;
use contracts::{HasConfig, HasPort, HasRegistry};
use std::sync::Arc;
use crate::shutdown::ShutdownHooks; // Import ShutdownHooks

// AppState needs to be Clone, but ShutdownHooks (Vec<Box<dyn ShutdownHookV2>>) is not easily Clone.
// We can wrap ShutdownHooks in an Arc to make AppState Clone.
// Alternatively, ShutdownHooks are only used at the end, so maybe they don't need to be in AppState,
// but rather passed to the shutdown_signal function.
// The prompt implies putting them into a "unified shutdown process," which might mean AppState is a good owner.

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<crate::config::Config>,
    pub registry: Arc<prometheus::Registry>,
    pub user_repo: Arc<dyn UserRepository>,
    pub shutdown_hooks: Arc<ShutdownHooks>, // Store hooks in an Arc
}

impl HasConfig for AppState {
    type Cfg = crate::config::Config;
    fn config(&self) -> &Self::Cfg {
        &self.config
    }
}

impl HasRegistry for AppState {
    fn registry(&self) -> &prometheus::Registry {
        &self.registry
    }
}

impl HasPort for AppState {
    fn port(&self) -> u16 {
        self.config.port
    }
}

impl contracts::HasUserRepository for AppState {
    fn user_repo(&self) -> Arc<dyn UserRepository> {
        self.user_repo.clone()
    }
}
