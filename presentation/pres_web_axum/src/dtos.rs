use contracts::ports::User as DomainUser;
use serde::{Deserialize, Serialize};

// === Request DTOs ===

#[derive(Deserialize, Debug)]
pub struct CreateUserRequest {
    pub name: String,
}

// === Response DTOs ===

#[derive(Serialize, Debug)]
pub struct UserResponse {
    pub id: String, // Usually UUIDs are represented as strings in JSON
    pub name: String,
}

impl From<DomainUser> for UserResponse {
    fn from(domain_user: DomainUser) -> Self {
        UserResponse {
            id: domain_user.id.to_string(),
            name: domain_user.name,
        }
    }
}

// A generic success response for operations like create or update if no body is needed
#[derive(Serialize, Debug)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}
