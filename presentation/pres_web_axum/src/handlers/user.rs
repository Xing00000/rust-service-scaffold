use crate::{
    dtos::{CreateUserRequest, UserResponse},
    error::ApiError,
};
use application::{error::AppError, use_cases::create_user::CreateUserCmd};
use axum::{extract::State, Json};

pub async fn create_user_handler<S>(
    State(app_state): State<S>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, ApiError>
where
    S: application::use_cases::create_user::HasCreateUserUc + Send + Sync + 'static,
{
    tracing::info!("Creating user with name: {}", payload.name);

    let user = app_state
        .create_user_uc()
        .exec(CreateUserCmd { name: payload.name })
        .await
        .map_err(AppError::Domain)?;

    tracing::info!("User created with ID: {}", user.id);
    Ok(Json(UserResponse::from(user)))
}
