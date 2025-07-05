use contracts::ports::User as DomainUser;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct UserResponse {
    pub id: String,
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

#[derive(Serialize, Debug)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}
