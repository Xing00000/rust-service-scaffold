use axum::{routing::get, Extension, Router};
use serde_json::Value;
use std::{
    panic,
    sync::{Arc, Mutex},
};
use tokio::time::{sleep, Duration};
use tower_http::{request_id::RequestId, trace::TraceLayer};
use tracing::subscriber::with_default;
use tracing_futures::WithSubscriber;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};
// 步驟 1 & 2: 重新引入 Mutex 來序列化 panic hook 測試
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

// 步驟 3: 將 panic hook 測試改為同步的 #[test]
#[test]
fn test_panic_hook_logs_details() {
    // 步驟 3.1: 取得鎖
    let _guard = TEST_MUTEX.lock().unwrap();

    let writer = TestWriter::new();
    let writer_for_closure = writer.clone();
    let subscriber = Registry::default().with(EnvFilter::new("trace")).with(
        fmt::layer()
            .json()
            .with_writer(move || writer_for_closure.clone()),
    );

    // 步驟 3.2: 建立自己的 runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // 步驟 3.3: 設定本地日誌，並在裡面執行 runtime
    with_default(subscriber, || {
        rt.block_on(async {
            std::panic::set_hook(Box::new(axum_logging_service::telemetry::panic_hook));
            let app = Router::new().route(
                "/test_panic",
                get(axum_logging_service::handlers::panic_handler),
            );
            let task_handle = tokio::spawn(async move {
                let request = hyper::Request::builder()
                    .uri("/test_panic")
                    .body(axum::body::Body::empty())
                    .unwrap();
                use tower::ServiceExt;
                let _ = app.oneshot(request).await;
            });
            let result = task_handle.await;
            assert!(
                result.is_err(),
                "Spawned task should have panicked but did not."
            );
            sleep(Duration::from_millis(100)).await;
        });
    });

    let logs = writer.get_logs();
    assert!(logs.contains("PANIC_HOOK_ACTIVATED_SENTINEL_MESSAGE"));
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
        panic::set_hook(Box::new(axum_logging_service::telemetry::panic_hook));
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

// 步驟 3: 將另一個 panic hook 測試也改為同步的 #[test]
#[test]
fn test_global_panic_hook_logs_from_tokio_task() {
    // 步驟 3.1: 取得鎖
    let _guard = TEST_MUTEX.lock().unwrap();
    let writer = TestWriter::new();
    let writer_for_closure = writer.clone();
    let subscriber = Registry::default().with(EnvFilter::new("trace")).with(
        fmt::layer()
            .json()
            .with_writer(move || writer_for_closure.clone()),
    );

    // 步驟 3.2: 建立自己的 runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // 步驟 3.3: 設定本地日誌，並在裡面執行 runtime
    with_default(subscriber, || {
        rt.block_on(async {
            std::panic::set_hook(Box::new(axum_logging_service::telemetry::panic_hook));
            let task_handle = tokio::spawn(async {
                panic!("Panic from a detached tokio task for global hook test");
            });
            let result = task_handle.await;
            assert!(result.is_err(), "Spawned task did not panic as expected.");
            sleep(Duration::from_millis(100)).await;
        });
    });

    let logs = writer.get_logs();
    assert!(logs.contains("PANIC_HOOK_ACTIVATED_SENTINEL_MESSAGE"));
}
