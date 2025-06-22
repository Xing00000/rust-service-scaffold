use axum::{
    extract::Request,
    http::StatusCode, // HeaderMap import removed
    middleware::Next,
    response::Response,
};
use tracing::info;

// TODO: Consider moving secrets like JWT_SECRET to a configuration file or environment variables
// and load them into an application state or a dedicated configuration struct.
// const JWT_SECRET: &str = "your-secret-jwt-token"; // Example secret, DO NOT use in production

/// Middleware for authenticating requests.
///
/// This middleware currently logs the Authorization header if present.
/// It's a placeholder for actual JWT Bearer token validation.
pub async fn auth_middleware(
    // headers: HeaderMap, // Headers will be extracted from the request
    request: Request, // Removed mut from request
    next: Next,
) -> Result<Response, StatusCode> {
    info!("Entering auth_middleware");

    // Extract headers from the request
    let headers = request.headers();
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(header_value) => {
            info!("Authorization header found: {}", header_value);
            // TODO: Implement actual Bearer JWT validation here.
            // 1. Check if the header starts with "Bearer ".
            // 2. Parse the token.
            // 3. Validate the token (signature, expiration, claims).
            //    - The JWT secret should be loaded from configuration/app state.
            //    - Example:
            //      ```
            //      use jsonwebtoken::{decode, DecodingKey, Validation};
            //      use crate::AppState; // Assuming you have an AppState
            //      use axum::extract::State;
            //
            //      let app_state = request.extensions().get::<State<AppState>>()
            //          .expect("AppState not found in request extensions");
            //      let secret = &app_state.jwt_secret; // Or however you store your secret
            //
            //      if let Some(token) = header_value.strip_prefix("Bearer ") {
            //          let decoding_key = DecodingKey::from_secret(secret.as_ref());
            //          let validation = Validation::default(); // Customize as needed
            //          match decode::<Claims>(token, &decoding_key, &validation) {
            //              Ok(token_data) => {
            //                  info!("Token validated successfully for user: {:?}", token_data.claims.sub);
            //                  // You might want to add user information to request extensions
            //                  // request.extensions_mut().insert(token_data.claims);
            //              }
            //              Err(e) => {
            //                  info!("JWT validation error: {:?}", e);
            //                  // return Err(StatusCode::UNAUTHORIZED); // Or appropriate error response
            //              }
            //          }
            //      } else {
            //          info!("Authorization header is not a Bearer token");
            //          // return Err(StatusCode::UNAUTHORIZED);
            //      }
            //      ```
            // For now, we just log and pass through.
        }
        None => {
            info!("No Authorization header found in the request.");
            // Depending on the routes, you might want to deny access if no token is present.
            // For some public routes, this might be acceptable.
            // return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // Proceed to the next middleware or handler
    let response = next.run(request).await;
    info!("Exiting auth_middleware");
    Ok(response)
}

// Example Claims structure for JWT - define your own as needed
// use serde::{Deserialize, Serialize};
// #[derive(Debug, Serialize, Deserialize)]
// struct Claims {
//    sub: String, // Subject (usually user ID)
//    exp: usize,  // Expiration time
//    // ... other claims
// }

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        routing::get,
        Router,
    };
    use tower::ServiceExt; // for `oneshot`
    // HeaderMap is no longer directly used by auth_middleware's signature in tests, but tests might still construct headers.
    // axum::http::header is used for request construction.
    use axum::http::header;


    async fn test_handler() -> &'static str {
        "Hello, world!"
    }

    // Test setup: The wrapper must now match the signature that from_fn expects for auth_middleware
    // auth_middleware expects (Request, Next)
    // So, the wrapper itself is auth_middleware if no state is needed, or a wrapper that passes state.
    // from_fn(auth_middleware) is the direct way if auth_middleware matches.
    // The tests were using a wrapper to extract HeaderMap, which is what auth_middleware did internally.
    // Now that auth_middleware extracts headers from Request, the tests can call it directly via from_fn.

    fn app_with_middleware() -> Router {
        Router::new()
            .route("/", get(test_handler))
            .layer(axum::middleware::from_fn(auth_middleware)) // Use auth_middleware directly
    }

    // The old wrapper auth_middleware_wrapper is no longer needed here as auth_middleware's signature is compatible.
    // async fn auth_middleware_wrapper(
    //     request: Request,
    //     next: Next,
    // ) -> Result<Response, StatusCode> {
    //     // Headers are extracted inside auth_middleware now
    //     auth_middleware(request, next).await
    // }


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

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_with_bearer_header() {
        let app = app_with_middleware();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(header::AUTHORIZATION, "Bearer sometoken123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        // In a real scenario, you would assert that the token was processed
        // or user information was added to the request.
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

        assert_eq!(response.status(), StatusCode::OK);
        // Assert that it's handled gracefully, even if not "Bearer"
    }
}
