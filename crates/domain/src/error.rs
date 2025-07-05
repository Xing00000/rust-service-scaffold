use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Business rule violation: {0}")]
    BusinessRule(String),
    
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
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
        assert_eq!(error.to_string(), "Entity not found: User not found");
    }

    #[test]
    fn test_invalid_operation_error() {
        let error = DomainError::InvalidOperation("Invalid operation".to_string());
        assert!(matches!(error, DomainError::InvalidOperation(_)));
        assert_eq!(error.to_string(), "Invalid operation: Invalid operation");
    }

    #[test]
    fn test_business_rule_error() {
        let error = DomainError::BusinessRule("Business rule violation".to_string());
        assert!(matches!(error, DomainError::BusinessRule(_)));
        assert_eq!(error.to_string(), "Business rule violation: Business rule violation");
    }
}
