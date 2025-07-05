use application::error::AppError;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Extension,
};
use axum::{http::StatusCode, Json};
use contracts::ports::DomainError;
use contracts::ports::MetricsRegistry;
use prometheus::{Encoder, TextEncoder};
use serde::{Deserialize, Serialize};
use tower_http::request_id::RequestId;

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
        return Err(AppError::Domain(DomainError::ValidationError {
            message: "User triggered a bad request".to_string(),
        })
        .into());
    }

    tracing::info!(request_id = %request_id, "Request processing finished successfully.");
    Ok(format!("Hello, World! Your Request ID is: {}", request_id))
}

pub async fn test_error_handler() -> Result<&'static str, ApiError> {
    Err(AppError::Domain(DomainError::InvalidOperation {
        message: "This is a test error triggered from the /test_error route.".to_string(),
    })
    .into())
}

pub async fn info_handler() -> Json<BuildInfo> {
    let info = BuildInfo {
        build_timestamp: env!("VERGEN_BUILD_TIMESTAMP"),
        git_commit_hash: env!("VERGEN_GIT_SHA"),
        git_branch: env!("VERGEN_GIT_BRANCH"),
    };
    Json(info)
}

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
