use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// /healthz/live - Liveness Probe
pub async fn live_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

/// /healthz/ready - Readiness Probe
pub async fn ready_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ready" })))
}
