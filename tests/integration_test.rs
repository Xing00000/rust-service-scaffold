use axum::{routing::get, Extension, Router};
use axum_logging_service::infrastructure::{telemetry, web::handlers};
use serde_json::Value;
use std::{
    panic,
    sync::{Arc, Mutex},
};
use tokio::time::{sleep, Duration};
use tower::ServiceExt;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::subscriber::with_default;
use tracing_futures::WithSubscriber;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};
// 步驟 1 & 2: 重新引入 Mutex 來序列化 panic hook 測試
use axum::body::{to_bytes, Body};
use axum_logging_service::app::AppState;
use axum_logging_service::config::Config;
// ✅ 修正: 從 hyper use 語句中移除 Body
use hyper::{Request, StatusCode};
use once_cell::sync::Lazy;
static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Clone, Default)]
struct TestWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl TestWriter {
    fn new() -> Self {
        Self::default()
    }
    fn get_logs(&self) -> String {
        String::from_utf8_lossy(&self.buf.lock().unwrap()).to_string()
    }
}

impl std::io::Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.buf.lock().unwrap().flush()
    }
}

async fn test_handler(Extension(request_id_extension): Extension<RequestId>) -> String {
    let request_id = request_id_extension
        .header_value()
        .to_str()
        .unwrap_or("unknown")
        .to_string();
    tracing::info!(request_id = %request_id, "Test handler processing request");
    format!("Hello from test! Request ID: {}", request_id)
}

// 這個測試不涉及全域狀態，保持原樣
#[tokio::test]
async fn test_logging_with_request_id() {
    let writer = TestWriter::new();
    let writer_for_closure = writer.clone();

    let subscriber = Registry::default().with(EnvFilter::new("trace")).with(
        fmt::layer()
            .json()
            .with_writer(move || writer_for_closure.clone()),
    );

    async {
        let test_request_id = "test-id-123";
        let rid = RequestId::new(hyper::header::HeaderValue::from_static(test_request_id));
        let mut request = hyper::Request::builder()
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();
        request.extensions_mut().insert(rid);
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(TraceLayer::new_for_http());
        use tower::ServiceExt;
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        sleep(Duration::from_millis(50)).await;
    }
    .with_subscriber(subscriber)
    .await;

    let logs = writer.get_logs();
    assert!(logs.contains(r#""request_id":"test-id-123""#));
}

#[test]
fn test_panic_hook_logs_details() {
    // 取得鎖以確保測試串行執行
    let _guard = TEST_MUTEX.lock().unwrap();

    let writer = TestWriter::new();
    let writer_for_closure = writer.clone();
    let subscriber = Registry::default()
        .with(EnvFilter::new("trace")) // 確保捕獲所有級別的日誌
        .with(
            fmt::layer()
                .json() // 確保輸出是 JSON
                .with_writer(move || writer_for_closure.clone()),
        );

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // 在 tracing subscriber 的上下文中執行異步代碼
    with_default(subscriber, || {
        rt.block_on(async {
            // 設置自定義的 panic hook
            std::panic::set_hook(Box::new(telemetry::panic_hook));

            // 創建一個簡單的 app，只包含會 panic 的路由
            let app = Router::new().route("/test_panic", get(handlers::panic_handler));

            // 在一個新的 tokio 任務中發送請求，以模擬 Axum 的運行環境
            let task_handle = tokio::spawn(async move {
                let request = hyper::Request::builder()
                    .uri("/test_panic")
                    .body(axum::body::Body::empty())
                    .unwrap();
                // `oneshot` 會發送請求並等待響應
                let _ = app.oneshot(request).await;
            });

            // 等待任務完成。因為 handler 會 panic，所以這裡應該返回 Err
            let result = task_handle.await;
            assert!(
                result.is_err(),
                "Spawned task should have panicked but did not."
            );

            // 稍作等待，確保日誌有時間被處理和寫入
            sleep(Duration::from_millis(150)).await;
        });
    });

    // 獲取所有日誌
    let logs = writer.get_logs();

    // 添加調試輸出，這在 CI 環境中尤其有用
    if logs.is_empty() {
        panic!("FAILED: No logs were captured!");
    }
    println!("--- CAPTURED LOGS ---\n{}\n--- END LOGS ---", logs);

    // 解析日誌並進行精確斷言
    let mut panic_log_found = false;
    for line in logs.lines().filter(|l| !l.is_empty()) {
        let log_entry: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("Failed to parse log line as JSON: {}\nLine: {}", e, line));

        // 我們要找的是由 panic_hook 產生的日誌
        if log_entry["target"] == "panic" {
            panic_log_found = true;

            // 斷言日誌級別
            assert_eq!(
                log_entry["level"], "ERROR",
                "Panic log level should be ERROR"
            );

            // 斷言 panic 的消息負載
            let payload = log_entry["fields"]["payload"].as_str().unwrap();
            assert!(
                payload.contains("This is a test panic deliberately triggered"),
                "Log message should contain the panic payload"
            );

            // 斷言 panic 的位置信息
            let location = log_entry["fields"]["location"].as_str().unwrap();
            assert!(
                location.contains("src/infrastructure/web/handlers.rs"),
                "Log should contain the correct panic location"
            );

            break; // 找到後即可退出循環
        }
    }

    // 最終斷言，確保我們確實找到了目標日log
    assert!(
        panic_log_found,
        "The detailed panic log (target='panic') was not found in the captured logs."
    );
}

#[test]
fn unit_test_logging_module() {
    let _guard = TEST_MUTEX.lock().unwrap();

    let writer = TestWriter::new();
    let writer_for_closure = writer.clone();
    let subscriber = Registry::default().with(EnvFilter::new("trace")).with(
        fmt::layer()
            .json()
            .with_writer(move || writer_for_closure.clone()),
    );

    with_default(subscriber, || {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(telemetry::panic_hook));
        let _ = panic::catch_unwind(|| {
            panic!("this is a unit test panic");
        });
        panic::set_hook(original_hook);
    });

    let logs = writer.get_logs();
    assert!(
        !logs.is_empty(),
        "Panic hook should have produced logs, but none were found."
    );

    let mut panic_details_log_found = false;
    for line in logs.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let log_entry: Value = serde_json::from_str(line).expect(&format!(
            "Log line should be valid JSON. Failed on line: {}",
            line
        ));

        if let Some(message) = log_entry["fields"]["message"].as_str() {
            if message == "A panic occurred" {
                panic_details_log_found = true;

                assert_eq!(log_entry["level"], "ERROR", "Log level should be ERROR");
                assert_eq!(log_entry["target"], "panic", "Log target should be 'panic'");

                let payload = log_entry["fields"]["payload"].as_str().unwrap();
                assert!(
                    payload.contains("this is a unit test panic"),
                    "Log message should contain the panic payload"
                );

                // **修正**: 检查 location 字段是一个非空的字符串
                let location_field = &log_entry["fields"]["location"];
                assert!(
                    location_field.is_string(),
                    "Log 'location' field should be a string."
                );
                assert!(
                    !location_field.as_str().unwrap().is_empty(),
                    "Log 'location' field should not be empty."
                );

                break;
            }
        }
    }

    assert!(
        panic_details_log_found,
        "The detailed panic log ('A panic occurred') was not found."
    );
}

#[test]
fn test_global_panic_hook_logs_from_tokio_task() {
    // 取得鎖以確保測試串行執行
    let _guard = TEST_MUTEX.lock().unwrap();

    let writer = TestWriter::new();
    let writer_for_closure = writer.clone();
    let subscriber = Registry::default().with(EnvFilter::new("trace")).with(
        fmt::layer()
            .json()
            .with_writer(move || writer_for_closure.clone()),
    );

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // 在 tracing subscriber 的上下文中執行異步代碼
    with_default(subscriber, || {
        rt.block_on(async {
            // 設置自定義的 panic hook
            std::panic::set_hook(Box::new(telemetry::panic_hook));

            // 在一個新的 tokio 任務中直接觸發 panic
            let task_handle = tokio::spawn(async move {
                panic!("Panic from a detached tokio task for global hook test");
            });

            // 等待任務完成。因為任務會 panic，所以這裡應該返回 Err
            let result = task_handle.await;
            assert!(result.is_err(), "Spawned task did not panic as expected.");

            // 稍作等待，確保日誌有時間被處理和寫入
            sleep(Duration::from_millis(150)).await;
        });
    });

    // 獲取所有日誌
    let logs = writer.get_logs();

    // 添加調試輸出
    if logs.is_empty() {
        panic!("FAILED: No logs were captured!");
    }
    println!(
        "--- CAPTURED LOGS (global_panic_hook) ---\n{}\n--- END LOGS ---",
        logs
    );

    // 解析日誌並進行精確斷言
    let mut panic_log_found = false;
    for line in logs.lines().filter(|l| !l.is_empty()) {
        let log_entry: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("Failed to parse log line as JSON: {}\nLine: {}", e, line));

        // 尋找由 panic_hook 產生的日誌
        if log_entry["target"] == "panic" {
            panic_log_found = true;

            assert_eq!(
                log_entry["level"], "ERROR",
                "Panic log level should be ERROR"
            );

            // 斷言 panic 的消息負載
            let payload = log_entry["fields"]["payload"].as_str().unwrap();
            assert!(
                payload.contains("Panic from a detached tokio task for global hook test"),
                "Log message should contain the correct panic payload"
            );

            // 斷言 panic 的位置信息
            // 這次 panic 發生在測試文件自身
            let location = log_entry["fields"]["location"].as_str().unwrap();
            assert!(
                location.contains("tests/integration_test.rs"),
                "Log should contain the correct panic location (in the test file)"
            );

            break;
        }
    }

    // 最終斷言
    assert!(
        panic_log_found,
        "The detailed panic log (target='panic') was not found in the captured logs."
    );
}

#[tokio::test]
async fn test_structured_error_response() {
    // Arrange
    let test_config = Arc::new(Config {
        port: 8080,
        log_level: "info".to_string(),
        otel_exporter_otlp_endpoint: "http://localhost:4317".to_string(),
        otel_service_name: "test-service".to_string(),
    });
    let app_state = AppState {
        config: test_config,
    };

    // ✅ 修正: 複製 main application 的 middleware stack
    // 這樣可以確保 `RequestId` extension 在 handler 中可用。
    let app = Router::new()
        .route("/", get(handlers::main_handler))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .with_state(app_state);

    // Act
    let request = Request::builder()
        .uri("/?make_error=true")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert
    // 現在這個斷言應該會成功，因為 handler 會被正確執行並返回 AppError::Validation
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // ✅ 修正: 為 `to_bytes` 函數提供一個合理的 body 大小限制（例如 64KB）。
    const BODY_LIMIT: usize = 65_536; // 64KB
    let body_bytes = to_bytes(response.into_body(), BODY_LIMIT).await.unwrap();

    let body_json: Value =
        serde_json::from_slice(&body_bytes).expect("Response body should be valid JSON");

    assert_eq!(body_json["error"]["code"], "VALIDATION");
    assert_eq!(
        body_json["error"]["message"],
        "Validation error: User triggered a bad request"
    );
}
