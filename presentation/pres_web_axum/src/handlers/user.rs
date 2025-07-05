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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dtos::CreateUserRequest;
    use application::use_cases::create_user::{CreateUserUseCase, HasCreateUserUc};
    use async_trait::async_trait;
    use contracts::ports::{DomainError, User};
    use std::sync::Arc;
    use uuid::Uuid;

    struct MockAppState;

    #[async_trait]
    impl CreateUserUseCase for MockAppState {
        async fn exec(&self, _cmd: CreateUserCmd) -> Result<User, DomainError> {
            Ok(User {
                id: Uuid::new_v4(),
                name: "Test User".to_string(),
            })
        }
    }

    impl HasCreateUserUc for MockAppState {
        fn create_user_uc(&self) -> Arc<dyn CreateUserUseCase> {
            Arc::new(MockAppState)
        }
    }

    #[tokio::test]
    async fn test_create_user_handler_success() {
        let app_state = MockAppState;
        let request = CreateUserRequest {
            name: "John Doe".to_string(),
        };

        let result = create_user_handler(axum::extract::State(app_state), Json(request)).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.name, "Test User");
    }
}
