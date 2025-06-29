use application::ports::UserRepository;
use contracts::{HasConfig, HasPort, HasRegistry};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    // 仍用 Arc 便於 clone，但對外 trait 回傳 &T
    pub config: Arc<crate::config::Config>,
    pub registry: Arc<prometheus::Registry>,
    pub user_repo: Arc<dyn UserRepository>,
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
