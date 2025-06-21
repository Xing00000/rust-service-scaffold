// src/infrastructure/web/handlers.rs

use crate::app::AppState;
use crate::error::AppError;
use axum::http::StatusCode;
use axum::Json;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Extension,
};
use prometheus::{Encoder, TextEncoder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::request_id::RequestId;

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
pub async fn panic_handler() -> Result<impl IntoResponse, AppError> {
    panic!("This is a test panic deliberately triggered from the /test_panic route!");
    Ok("This response will never be sent.")
}
pub async fn metrics_handler() -> impl IntoResponse {
    // ✅ 不再需要 State
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();

    // ✅ 從全局默認註冊表收集指標
    let metric_families = prometheus::gather();

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
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
