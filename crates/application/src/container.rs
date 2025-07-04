use std::sync::Arc;

use contracts::ports::{DynObservability, DynUserRepo};

use crate::use_cases::create_user::{CreateUserUseCase, UserSvc};

/// 依賴注入容器
pub struct Container {
    user_repo: DynUserRepo,
    observability: DynObservability,
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

/// 提供可觀測性的 trait
pub trait HasObservability {
    fn observability(&self) -> contracts::ports::DynObservability;
}

impl HasObservability for Container {
    fn observability(&self) -> contracts::ports::DynObservability {
        self.observability.clone()
    }
}

/// 提供儲存庫的 trait (內部使用)
pub trait HasUserRepo {
    fn user_repo(&self) -> contracts::ports::DynUserRepo;
}

impl HasUserRepo for Container {
    fn user_repo(&self) -> contracts::ports::DynUserRepo {
        self.user_repo.clone()
    }
}
