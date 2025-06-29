// app/src/state.rs
use contracts::{HasConfig, HasPort, HasRegistry};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<crate::config::Config>,
    pub registry: Arc<prometheus::Registry>,
}

impl HasConfig for AppState {
    type Cfg = crate::config::Config;
    fn config(&self) -> &Arc<Self::Cfg> {
        &self.config
    }
}
impl HasRegistry for AppState {
    fn registry(&self) -> &Arc<prometheus::Registry> {
        &self.registry
    }
}
// app/app_state.rs
impl HasPort for AppState {
    fn port(&self) -> u16 {
        self.config.port
    }
}
