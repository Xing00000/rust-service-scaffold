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
        tracing::warn!(request_id = %request_id, "Simulating a bad request error.");
        return Err(AppError::BadRequest(
            "User triggered a bad request".to_string(),
        ));
    }

    tracing::info!(request_id = %request_id, "Request processing finished successfully.");
    Ok(format!("Hello, World! Your Request ID is: {}", request_id))
}

pub async fn test_error_handler() -> Result<&'static str, AppError> {
    Err(AppError::InternalServerError)
}

/// 觸發 panic 的處理函數，對應 `/test_panic` 端點。
///
/// 這個函數故意觸發一個 panic，用於測試全局 panic hook 是否能
/// 捕獲 panic、記錄詳細信息並優雅地終止進程。
///
/// 我們使用 `#[allow(unreachable_code)]` 來抑制編譯器關於 panic! 之後
/// 代碼不可達的警告，並返回一個 Result 以符合未來的 Rust 版本要求。
#[allow(unreachable_code)]
pub async fn panic_handler() -> Result<impl IntoResponse, AppError> {
    panic!("This is a test panic deliberately triggered from the /test_panic route!");

    // 這段代碼實際上永遠不會被執行，但它的存在是為了滿足類型簽名，
    // 從而解決 Rust 2024 中關於 "never type fallback" 的編譯問題。
    Ok("This response will never be sent.")
}
