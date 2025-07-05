use crate::{error::DomainError, id::UserId};

//=== Domain Entity ===//
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub name: String,
}

impl User {
    /// 創建新用戶（包含業務規則驗證）
    pub fn new(id: UserId, name: String) -> Result<Self, DomainError> {
        Self::validate_name(&name)?;
        Ok(Self { id, name })
    }

    /// 更新用戶名稱
    pub fn update_name(&mut self, new_name: String) -> Result<(), DomainError> {
        Self::validate_name(&new_name)?;
        self.name = new_name;
        Ok(())
    }

    /// 驗證用戶名稱的業務規則
    fn validate_name(name: &str) -> Result<(), DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError {
                message: "User name cannot be empty".to_string(),
            });
        }

        if name.len() > 100 {
            return Err(DomainError::ValidationError {
                message: "User name cannot exceed 100 characters".to_string(),
            });
        }

        if name.chars().any(|c| c.is_control()) {
            return Err(DomainError::ValidationError {
                message: "User name cannot contain control characters".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation_success() {
        let id = UserId::from_string("test-id".to_string());
        let name = "Test User".to_string();

        let user = User::new(id.clone(), name.clone()).unwrap();

        assert_eq!(user.id, id);
        assert_eq!(user.name, name);
    }

    #[test]
    fn test_user_creation_empty_name() {
        let id = UserId::from_string("test-id".to_string());
        let result = User::new(id, "".to_string());

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError { message } => {
                assert!(message.contains("empty"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_user_creation_long_name() {
        let id = UserId::from_string("test-id".to_string());
        let long_name = "a".repeat(101);
        let result = User::new(id, long_name);

        assert!(result.is_err());
    }

    #[test]
    fn test_user_update_name_success() {
        let id = UserId::from_string("test-id".to_string());
        let mut user = User::new(id, "Original".to_string()).unwrap();

        let result = user.update_name("Updated".to_string());
        assert!(result.is_ok());
        assert_eq!(user.name, "Updated");
    }

    #[test]
    fn test_user_clone() {
        let id = UserId::from_string("test-id".to_string());
        let user = User::new(id.clone(), "Original".to_string()).unwrap();

        let cloned = user.clone();
        assert_eq!(user.id, cloned.id);
        assert_eq!(user.name, cloned.name);
    }
}
