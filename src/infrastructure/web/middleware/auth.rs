use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use tracing::{info, warn};

use crate::app::AppState;

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

/// Middleware for authenticating requests using JWT.
pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    info!("Entering auth_middleware");

    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header.and_then(|h| h.strip_prefix("Bearer ")) {
        Some(t) => t,
        None => {
            warn!("Missing or malformed Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let secret = app_state.config.jwt_secret.as_bytes();
    match decode::<Claims>(token, &DecodingKey::from_secret(secret), &Validation::default()) {
        Ok(token_data) => {
            request.extensions_mut().insert(token_data.claims);
            let response = next.run(request).await;
            info!("Exiting auth_middleware");
            Ok(response)
        }
        Err(err) => {
            warn!(?err, "JWT validation error");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        routing::get,
        Router,
    };
    use tower::ServiceExt; // for `oneshot`
    use axum::http::header;


    async fn test_handler() -> &'static str {
        "Hello, world!"
    }


    fn app_with_middleware() -> Router {
        use crate::{app::AppState, config::Config};
        use std::sync::Arc;

        let app_state = AppState {
            config: Arc::new(Config {
                port: 0,
                log_level: "info".into(),
                otel_exporter_otlp_endpoint: "http://localhost".into(),
                otel_service_name: "test".into(),
                rate_limit_per_second: 1,
                rate_limit_burst_size: 1,
                http_headers: None,
                jwt_secret: "secret".into(),
            }),
            registry: Arc::new(prometheus::Registry::new()),
        };

        Router::new()
            .route("/", get(test_handler))
            .layer(axum::middleware::from_fn_with_state(app_state.clone(), auth_middleware))
            .with_state(app_state)
    }



    #[tokio::test]
    async fn test_auth_middleware_no_header() {
        let app = app_with_middleware();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_with_bearer_header() {
        let app = app_with_middleware();

        let token = {
            use jsonwebtoken::{encode, EncodingKey, Header};
            use std::time::{SystemTime, UNIX_EPOCH};

            let exp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize
                + 60;
            let claims = Claims {
                sub: "tester".into(),
                exp,
            };
            encode(&Header::default(), &claims, &EncodingKey::from_secret(b"secret"))
                .unwrap()
        };

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_with_non_bearer_header() {
        let app = app_with_middleware();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(header::AUTHORIZATION, "Basic somecredentials")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
