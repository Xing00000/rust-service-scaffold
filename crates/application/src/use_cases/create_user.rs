use std::sync::Arc;

use async_trait::async_trait;
use contracts::{
    ports::{User, UserRepository},
    DomainError,
};
use crate::id_service::IdService;

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
        // 1) 生成 ID
        let user_id = IdService::generate_user_id();
        
        // 2) 建立 Domain 物件（使用 Domain 層的業務驗證）
        let user = User::new(user_id, cmd.name)?;

        // 3) 儲存
        self.repo.save(&user).await?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contracts::{User, UserId, DomainError};
    use std::{sync::Arc, future::Future, pin::Pin};
    use domain::UserRepository;

    struct MockUserRepository;

    impl UserRepository for MockUserRepository {
        fn find(&self, _id: &UserId) -> Pin<Box<dyn Future<Output = Result<User, DomainError>> + Send + '_>> {
            Box::pin(async { Err(DomainError::NotFound { message: "Not implemented".to_string() }) })
        }

        fn save(&self, _user: &User) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }

        fn shutdown(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
            Box::pin(async {})
        }
    }

    #[tokio::test]
    async fn test_create_user_success() {
        // Arrange
        let mock_repo = MockUserRepository;
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
        let mock_repo = MockUserRepository;
        let use_case = UserSvc::new(Arc::new(mock_repo));
        let cmd = CreateUserCmd {
            name: "".to_string(),
        };

        // Act
        let result = use_case.exec(cmd).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError { message } => assert!(message.contains("empty")),
            _ => panic!("Expected validation error"),
        }
    }
}
