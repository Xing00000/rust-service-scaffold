use application::error::AppError;

use application::use_cases::create_user::CreateUserCmd;
use axum::http::StatusCode;
use axum::Json;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Extension,
};
use contracts::ports::{DomainError, MetricsRegistry};
use prometheus::{Encoder, TextEncoder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::request_id::RequestId;
// For new user ID generation

use crate::dtos::{CreateUserRequest, UserResponse}; // Import DTOs
use crate::error::ApiError;

#[derive(Deserialize)]
pub struct HandlerParams {
    make_error: Option<bool>,
}

#[derive(Serialize)]
pub struct BuildInfo {
    build_timestamp: &'static str,
    git_commit_hash: &'static str,
    git_branch: &'static str,
}

pub async fn main_handler<S>(
    State(_app_state): State<S>,
    Extension(request_id_extension): Extension<RequestId>,
    Query(params): Query<HandlerParams>,
) -> Result<String, ApiError>
where
    S: Send + Sync + 'static,
{
    let request_id = request_id_extension
        .header_value()
        .to_str()
        .unwrap_or("unknown");

    tracing::info!(
        request_id = %request_id,
        "Processing request for the main handler"
    );

    if params.make_error.unwrap_or(false) {
        tracing::warn!(request_id = %request_id, "Simulating a validation error.");

        // ✅ [關鍵修正]: 確保此處返回的是 `ApiError::Validation`
        // 這將在 `IntoResponse` 中被正確地映射為 HTTP 422。
        return Err(AppError::Domain(DomainError::Validation(
            "User triggered a bad request".to_string(),
        ))
        .into());
    }

    tracing::info!(request_id = %request_id, "Request processing finished successfully.");
    Ok(format!("Hello, World! Your Request ID is: {}", request_id))
}

// 這個 handler 返回 `ApiError::Internal`，對應 HTTP 500。
pub async fn test_error_handler() -> Result<&'static str, ApiError> {
    Err(AppError::Domain(DomainError::Unexpected(
        "This is a test error triggered from the /test_error route.".to_string(),
    ))
    .into())
}

// === Health Check Handlers ===

/// /healthz/live - Liveness Probe
///
/// 用於確認服務進程正在運行。只要服務啟動，就應該返回 200 OK。
pub async fn live_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

/// /healthz/ready - Readiness Probe
///
/// 用於確認服務已準備好接收流量。未來可以擴展以檢查數據庫連接等。
pub async fn ready_handler() -> impl IntoResponse {
    // 現在，它和 live_handler 一樣。
    // 在未來，你可以在這裡添加檢查，例如：
    // if !db_pool.is_connected() { return StatusCode::SERVICE_UNAVAILABLE; }
    (StatusCode::OK, Json(json!({ "status": "ready" })))
}

/// /info - Build Information Endpoint
///
/// 使用 vergen 在編譯時注入的構建和 Git 信息。
pub async fn info_handler() -> Json<BuildInfo> {
    let info = BuildInfo {
        build_timestamp: env!("VERGEN_BUILD_TIMESTAMP"),
        git_commit_hash: env!("VERGEN_GIT_SHA"),
        git_branch: env!("VERGEN_GIT_BRANCH"),
    };
    Json(info)
}

/// 觸發 panic 的處理函數。
#[allow(unreachable_code)]
pub async fn panic_handler() -> Result<impl IntoResponse, ApiError> {
    panic!("This is a test panic deliberately triggered from the /test_panic route!");
    Ok("This response will never be sent.")
}

pub async fn metrics_handler<S>(State(app_state): State<S>) -> impl IntoResponse
where
    S: MetricsRegistry + Send + Sync + 'static,
{
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();

    if let Err(e) = encoder.encode(&app_state.registry().gather(), &mut buffer) {
        tracing::error!("Failed to encode prometheus metrics: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to encode metrics: {}", e),
        )
            .into_response()
    } else {
        (
            StatusCode::OK,
            [("Content-Type", prometheus::TEXT_FORMAT)],
            buffer,
        )
            .into_response()
    }
}

// === User Handlers ===

pub async fn create_user_handler<S>(
    State(app_state): State<S>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, ApiError>
where
    S: application::use_cases::create_user::HasCreateUserUc + Send + Sync + 'static,
{
    tracing::info!("Attempting to create user with name: {}", payload.name);

    let user = app_state
        .create_user_uc()
        .exec(CreateUserCmd { name: payload.name })
        .await
        .map_err(AppError::Domain)?; // Map DomainError from repo to AppError

    tracing::info!("User created successfully with ID: {}", user.id);
    Ok(Json(UserResponse::from(user)))
}
