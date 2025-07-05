use std::time::Instant;

use axum::body::Body;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Extension;
use contracts::DynObservability; // Added for ObservabilityPort

// 把 ObservabilityPort trait object 傳進來 (可透過 extension or app_state)
pub async fn axum_metrics_middleware(
    Extension(obs): Extension<DynObservability>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    // 先把 method、path 抓出來 clone 成 String
    let method = req.method().as_str().to_owned();
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_owned())
        .unwrap_or_else(|| req.uri().path().to_owned());

    // Use the ObservabilityPort trait object
    obs.on_request_start(&method, &path).await;

    let start = Instant::now();

    // 這時 req 沒有任何借用，可以直接 move
    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();

    // Use the ObservabilityPort trait object
    obs.on_request_end(&method, &path, status, latency).await;

    response
}
