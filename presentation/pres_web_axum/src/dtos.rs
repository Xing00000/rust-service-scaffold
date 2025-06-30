use serde::{Deserialize, Serialize};
use domain::user::User as DomainUser; // Alias to avoid confusion
use uuid::Uuid;

// === Request DTOs ===

#[derive(Deserialize, Debug)]
pub struct CreateUserRequest {
    pub name: String,
}

// For path parameters, like /users/{id}
#[derive(Deserialize, Debug)]
pub struct GetUserPath {
    pub id: Uuid, // Axum can parse Uuid from path directly
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
