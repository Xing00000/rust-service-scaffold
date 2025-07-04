use application::{Container, HasCreateUserUc};
use contracts::ports::MetricsRegistry;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<crate::config::Config>,
    pub registry: Arc<prometheus::Registry>,
    pub container: Arc<Container>,
}

impl MetricsRegistry for AppState {
    fn registry(&self) -> &prometheus::Registry {
        &self.registry
    }
}

impl application::use_cases::create_user::HasCreateUserUc for AppState {
    fn create_user_uc(&self) -> Arc<dyn application::use_cases::create_user::CreateUserUseCase> {
        self.container.create_user_uc()
    }
}
