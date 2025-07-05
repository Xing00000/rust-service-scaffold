//=== Pure Domain Errors (Zero External Dependencies) ===//

#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    BusinessRule { message: String },
    NotFound { message: String },
    InvalidOperation { message: String },
    ValidationError { message: String },
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::BusinessRule { message } => write!(f, "Business rule violation: {}", message),
            DomainError::NotFound { message } => write!(f, "Entity not found: {}", message),
            DomainError::InvalidOperation { message } => write!(f, "Invalid operation: {}", message),
            DomainError::ValidationError { message } => write!(f, "Validation error: {}", message),
        }
    }
}

impl std::error::Error for DomainError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let error = DomainError::ValidationError { message: "Invalid input".to_string() };
        assert!(matches!(error, DomainError::ValidationError { .. }));
        assert_eq!(error.to_string(), "Validation error: Invalid input");
    }

    #[test]
    fn test_not_found_error() {
        let error = DomainError::NotFound { message: "User not found".to_string() };
        assert!(matches!(error, DomainError::NotFound { .. }));
        assert_eq!(error.to_string(), "Entity not found: User not found");
    }

    #[test]
    fn test_invalid_operation_error() {
        let error = DomainError::InvalidOperation { message: "Invalid operation".to_string() };
        assert!(matches!(error, DomainError::InvalidOperation { .. }));
        assert_eq!(error.to_string(), "Invalid operation: Invalid operation");
    }

    #[test]
    fn test_business_rule_error() {
        let error = DomainError::BusinessRule { message: "Business rule violation".to_string() };
        assert!(matches!(error, DomainError::BusinessRule { .. }));
        assert_eq!(
            error.to_string(),
            "Business rule violation: Business rule violation"
        );
    }

    #[test]
    fn test_error_clone_and_equality() {
        let error1 = DomainError::ValidationError { message: "test".to_string() };
        let error2 = error1.clone();
        assert_eq!(error1, error2);
    }
}
