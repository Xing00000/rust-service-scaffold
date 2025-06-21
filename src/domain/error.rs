// src/domain/error.rs
// 定義只與業務邏輯相關的錯誤。

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Validation failed: {0}")]
    ValidationError(String),
    // 例如：
    // #[error("User '{0}' not found")]
    // UserNotFound(String),
}
