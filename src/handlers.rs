// src/handlers.rs

// Handler function to trigger a panic
pub async fn panic_handler() -> &'static str {
    panic!("This is a test panic from the /test_panic route!");
}

// We can also move other handlers here if desired, e.g., test_error_handler, handler
// For now, just panic_handler as per immediate need.
// pub use crate::error::AppError; // If other handlers are moved that use AppError

// Example of moving another handler (if needed later):
// use axum::extract::Query;
// use axum_extra::extract::RequestId;
// use opentelemetry::{Context as OtelContext, KeyValue};
// use serde::Deserialize;

// #[derive(Deserialize)]
// pub struct HandlerParams {
//     pub make_error: Option<bool>,
// }

// pub async fn main_handler(
//     Extension(request_id_extension): Extension<RequestId>,
//     Query(params): Query<HandlerParams>,
// ) -> Result<String, AppError> {
//     let request_id = request_id_extension
//         .header_value()
//         .to_str()
//         .unwrap_or("unknown")
//         .to_string();

//     if params.make_error.unwrap_or(false) {
//         tracing::info!("Simulating an error for request_id: {}", request_id);
//         return Err(AppError::BadRequest(format!("Triggered error for request_id: {}", request_id)));
//     }

//     let current_otel_cx = OtelContext::current();
//     let _current_otel_span = current_otel_cx.span();

//     let greeting_message = "Hello, World!";
//     let response_body = {
//         let _custom_work_span_guard = tracing::info_span!(
//             "custom_work_in_handler",
//             service_operation = "generate_greeting",
//             request_id = %request_id
//         ).entered();
//         tracing::info!("Performing custom work: generating greeting message for request_id: {}", request_id);
//         format!("{} Request ID: {}", greeting_message, request_id)
//     };
//     tracing::info!("Request processing finished for request_id: {}", request_id);
//     Ok(response_body)
// }
