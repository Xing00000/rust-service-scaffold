use std::sync::Arc;

use async_trait::async_trait;
use contracts::{
    ports::{User, UserRepository},
    DomainError,
};
use uuid::{timestamp::context, Timestamp, Uuid};

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

#[cfg(test)]
mod tests {
    use super::*;
    use contracts::ports::MockUserRepository;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_user_success() {
        // Arrange
        let mut mock_repo = MockUserRepository::new();
        mock_repo
            .expect_save()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let use_case = UserSvc::new(Arc::new(mock_repo));
        let cmd = CreateUserCmd {
            name: "Test User".to_string(),
        };

        // Act
        let result = use_case.exec(cmd).await;

        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Test User");
    }

    #[tokio::test]
    async fn test_create_user_empty_name() {
        // Arrange
        let mock_repo = MockUserRepository::new();
        let use_case = UserSvc::new(Arc::new(mock_repo));
        let cmd = CreateUserCmd {
            name: "".to_string(),
        };

        // Act
        let result = use_case.exec(cmd).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::Validation(msg) => assert_eq!(msg, "name cannot be empty"),
            _ => panic!("Expected validation error"),
        }
    }
}
