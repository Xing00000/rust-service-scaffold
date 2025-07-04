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
            .returning(|_| Ok(()));

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