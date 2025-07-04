use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Duplicate entry: {0}")]
    Duplicate(String),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let error = DomainError::Validation("Invalid input".to_string());
        assert!(matches!(error, DomainError::Validation(_)));
        assert_eq!(error.to_string(), "Validation error: Invalid input");
    }

    #[test]
    fn test_not_found_error() {
        let error = DomainError::NotFound("User not found".to_string());
        assert!(matches!(error, DomainError::NotFound(_)));
        assert_eq!(error.to_string(), "Not found: User not found");
    }

    #[test]
    fn test_duplicate_error() {
        let error = DomainError::Duplicate("User already exists".to_string());
        assert!(matches!(error, DomainError::Duplicate(_)));
        assert_eq!(error.to_string(), "Duplicate entry: User already exists");
    }

    #[test]
    fn test_unexpected_error() {
        let error = DomainError::Unexpected("System failure".to_string());
        assert!(matches!(error, DomainError::Unexpected(_)));
        assert_eq!(error.to_string(), "Unexpected error: System failure");
    }
}
