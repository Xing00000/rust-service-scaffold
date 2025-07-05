use std::collections::HashMap;
use std::sync::Arc;
use std::any::{Any, TypeId};

use contracts::ports::{DynObservability, DynUserRepo};
use crate::use_cases::create_user::{CreateUserUseCase, UserSvc};

/// 改進的依賴注入容器
pub struct Container {
    // 基礎設施依賴
    user_repo: DynUserRepo,
    observability: DynObservability,
    
    // 用例註冊表
    use_cases: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Container {
    pub fn new(user_repo: DynUserRepo, observability: DynObservability) -> Self {
        let mut container = Self {
            user_repo: user_repo.clone(),
            observability,
            use_cases: HashMap::new(),
        };
        
        // 註冊預設用例
        let create_user_uc: Arc<dyn CreateUserUseCase> = Arc::new(UserSvc::new(user_repo));
        container.register_use_case(create_user_uc);
        
        container
    }
    
    /// 註冊用例
    pub fn register_use_case<T>(&mut self, use_case: Arc<T>) 
    where 
        T: Send + Sync + 'static + ?Sized 
    {
        self.use_cases.insert(TypeId::of::<T>(), Box::new(use_case));
    }
    
    /// 獲取用例
    pub fn get_use_case<T>(&self) -> Option<Arc<T>>
    where 
        T: Send + Sync + 'static + ?Sized 
    {
        self.use_cases.get(&TypeId::of::<T>())?
            .downcast_ref::<Arc<T>>()
            .cloned()
    }
}

/// 提供用例的 trait
pub trait HasCreateUserUc {
    fn create_user_uc(&self) -> Arc<dyn CreateUserUseCase>;
}

impl HasCreateUserUc for Container {
    fn create_user_uc(&self) -> Arc<dyn CreateUserUseCase> {
        self.get_use_case::<dyn CreateUserUseCase>()
            .expect("CreateUserUseCase not registered")
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
