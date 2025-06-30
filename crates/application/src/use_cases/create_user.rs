use std::sync::Arc;

use async_trait::async_trait;
use domain::{error::DomainError, user::User};
use uuid::{timestamp::context, Timestamp, Uuid};

use crate::ports::UserRepository;

#[derive(Debug)]
pub struct CreateUserCmd {
    pub name: String,
}
pub trait HasCreateUserUc: Send + Sync {
    fn create_user_uc(&self) -> Arc<dyn CreateUserUseCase>;
}

#[async_trait]
pub trait CreateUserUseCase: Send + Sync {
    async fn exec(&self, cmd: CreateUserCmd) -> Result<User, DomainError>;
}

// 具體實作

pub struct UserSvc {
    repo: Arc<dyn UserRepository>,
}

impl UserSvc {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl CreateUserUseCase for UserSvc {
    async fn exec(&self, cmd: CreateUserCmd) -> Result<User, DomainError> {
        // 1) 業務驗證
        if cmd.name.trim().is_empty() {
            return Err(DomainError::Validation("name cannot be empty".into()));
        }

        // 2) 建立 Domain 物件
        let user = User {
            id: Uuid::new_v7(Timestamp::now(context::ContextV7::new())),
            name: cmd.name,
        };

        // 3) 儲存
        self.repo.save(&user).await?;

        Ok(user)
    }
}
