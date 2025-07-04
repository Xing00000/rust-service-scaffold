use std::sync::Arc;

use contracts::ports::{DynObservability, DynUserRepo};

use crate::use_cases::create_user::{CreateUserUseCase, UserSvc};

/// 依賴注入容器
pub struct Container {
    pub user_repo: DynUserRepo,
    pub observability: DynObservability,
    pub create_user_uc: Arc<dyn CreateUserUseCase>,
}

impl Container {
    pub fn new(user_repo: DynUserRepo, observability: DynObservability) -> Self {
        let create_user_uc: Arc<dyn CreateUserUseCase> = Arc::new(UserSvc::new(user_repo.clone()));

        Self {
            user_repo,
            observability,
            create_user_uc,
        }
    }
}

/// 提供用例的 trait
pub trait HasCreateUserUc {
    fn create_user_uc(&self) -> Arc<dyn CreateUserUseCase>;
}

impl HasCreateUserUc for Container {
    fn create_user_uc(&self) -> Arc<dyn CreateUserUseCase> {
        self.create_user_uc.clone()
    }
}
