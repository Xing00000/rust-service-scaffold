// src/infrastructure/web/handlers.rs

use crate::app::AppState;
use crate::error::AppError;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Extension,
};
use serde::Deserialize;
use tower_http::request_id::RequestId;

#[derive(Deserialize)]
pub struct HandlerParams {
    make_error: Option<bool>,
}

pub async fn main_handler(
    State(app_state): State<AppState>,
    Extension(request_id_extension): Extension<RequestId>,
    Query(params): Query<HandlerParams>,
) -> Result<String, AppError> {
    let request_id = request_id_extension
        .header_value()
        .to_str()
        .unwrap_or("unknown");

    tracing::info!(
        request_id = %request_id,
        app_port = %app_state.config.port,
        "Processing request for the main handler"
    );

    if params.make_error.unwrap_or(false) {
        tracing::warn!(request_id = %request_id, "Simulating a validation error.");

        // ✅ [關鍵修正]: 確保此處返回的是 `AppError::Validation`
        // 這將在 `IntoResponse` 中被正確地映射為 HTTP 422。
        return Err(AppError::Validation(
            "User triggered a bad request".to_string(),
        ));
    }

    tracing::info!(request_id = %request_id, "Request processing finished successfully.");
    Ok(format!("Hello, World! Your Request ID is: {}", request_id))
}

// 這個 handler 返回 `AppError::Internal`，對應 HTTP 500。
pub async fn test_error_handler() -> Result<&'static str, AppError> {
    Err(AppError::Internal)
}

/// 觸發 panic 的處理函數。
#[allow(unreachable_code)]
pub async fn panic_handler() -> Result<impl IntoResponse, AppError> {
    panic!("This is a test panic deliberately triggered from the /test_panic route!");
    Ok("This response will never be sent.")
}
