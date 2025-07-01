use axum::{routing::get, Extension, Router};
use bootstrap::{
    config::{self, Config},
    state::AppState,
};

use domain::{error::DomainError, user::User};
use infra_telemetry::telemetry;
use pres_web_axum::handlers;
use serde_json::Value;
use std::{
    panic,
    sync::{Arc, Mutex},
};
use tokio::time::{sleep, Duration};
use tower::ServiceExt; // For `oneshot`
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::subscriber::with_default;
use tracing_futures::WithSubscriber;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};
// 步驟 1 & 2: 重新引入 Mutex 來序列化 panic hook 測試
use application::use_cases::create_user::{CreateUserUseCase, UserSvc};
use axum::body::{to_bytes, Body};

use application::ports::UserRepository;
use async_trait::async_trait;
use hyper::{Request, StatusCode};
use once_cell::sync::Lazy;
use uuid::Uuid;

// For FakeObs
use application::ports::{DynObs, ObservabilityPort};
use axum::middleware;
use pres_web_axum::middleware::telemetry_middleware;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Clone, Default)]
struct FakeObs {
    request_start_calls: Arc<AtomicUsize>,
    request_end_calls: Arc<AtomicUsize>,
}

impl FakeObs {
    fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)] // Potentially used in future assertions
    fn get_request_start_calls(&self) -> usize {
        self.request_start_calls.load(Ordering::SeqCst)
    }

    #[allow(dead_code)] // Potentially used in future assertions
    fn get_request_end_calls(&self) -> usize {
        self.request_end_calls.load(Ordering::SeqCst)
    }

    #[allow(dead_code)] // Potentially used in future assertions
    fn reset_counts(&self) {
        self.request_start_calls.store(0, Ordering::SeqCst);
        self.request_end_calls.store(0, Ordering::SeqCst);
    }
}

#[async_trait]
impl ObservabilityPort for FakeObs {
    async fn on_request_start(&self, _method: &str, _path: &str) {
        self.request_start_calls.fetch_add(1, Ordering::SeqCst);
    }

    async fn on_request_end(&self, _method: &str, _path: &str, _status: u16, _latency: f64) {
        self.request_end_calls.fetch_add(1, Ordering::SeqCst);
    }
}

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
                location.contains("presentation/pres_web_axum/src/handlers.rs"),
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

// 實作一個測試專用 fake/mock struct
#[derive(Default)]
pub struct FakeUserRepository;
#[async_trait]
impl UserRepository for FakeUserRepository {
    async fn find(&self, id: &Uuid) -> Result<User, DomainError> {
        // fake 行為
        Ok(User {
            id: *id,
            name: "Test".to_string(),
        })
    }
    async fn save(&self, _user: &User) -> Result<(), DomainError> {
        Ok(())
    }

    async fn shutdown(&self) {}
}

#[tokio::test]
async fn test_structured_error_response() {
    // Arrange
    let test_config = Arc::new(Config {
        port: 8080,
        log_level: "info".to_string(),
        otel_exporter_otlp_endpoint: "http://localhost:4317".to_string(),
        otel_service_name: "test-service".to_string(),
        rate_limit_per_second: 1,
        rate_limit_burst_size: 50,
        http_headers: Some(vec![config::HttpHeader {
            name: "X-Test-Header".to_string(),
            value: "TestValue".to_string(),
        }]),
        database_url: "postgres://user:password@localhost/test_db".to_string(),
        db_max_conn: 10,
    });
    let registry = prometheus::Registry::new();
    let mock_repo = Arc::new(FakeUserRepository::default());
    let create_user_uc: Arc<dyn CreateUserUseCase> = Arc::new(UserSvc::new(mock_repo.clone()));

    let fake_obs_instance = Arc::new(FakeObs::new());
    let obs_port_for_app_state: DynObs = fake_obs_instance.clone(); // Clone for AppState
    let obs_port_for_extension: DynObs = fake_obs_instance.clone(); // Clone for Extension layer

    let app_state = AppState {
        config: test_config.clone(),
        registry: Arc::new(registry),
        create_user_uc,
        obs_port: obs_port_for_app_state, // Add FakeObs to AppState
    };

    // ✅ 修正: 複製 main application 的 middleware stack
    // 這樣可以確保 `RequestId` extension 在 handler 中可用。
    // AND adding telemetry middleware with FakeObs
    let app = Router::new()
        .route("/", get(handlers::main_handler::<AppState>))
        .layer(
            // Layers from common_layers in app.rs, adapted for test
            tower::ServiceBuilder::new()
                .layer(axum::extract::Extension(obs_port_for_extension)) // Inject FakeObs via Extension
                .layer(middleware::from_fn(
                    telemetry_middleware::axum_metrics_middleware,
                ))
                .layer(TraceLayer::new_for_http())
                .layer(PropagateRequestIdLayer::x_request_id())
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid)), // Removed GovernorLayer for simplicity in this test
        )
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

    assert_eq!(body_json["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(
        body_json["error"]["message"],
        "Validation error: User triggered a bad request"
    );

    // Assert FakeObs calls
    assert_eq!(
        fake_obs_instance.get_request_start_calls(),
        1,
        "on_request_start should have been called once"
    );
    assert_eq!(
        fake_obs_instance.get_request_end_calls(),
        1,
        "on_request_end should have been called once"
    );
}

// Test for rate limiting
#[tokio::test]
async fn test_rate_limiting() {
    // use ::bootstrap::Application; // This was for a potential alternative way to test, not needed now
    use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

    // Configure a governor layer similar to the main application
    // We use a small burst size and short period for faster testing.
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .burst_size(2) // Allow 2 requests
            .period(std::time::Duration::from_secs(1)) // Per 1 second
            .finish()
            .unwrap(),
    );

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid)) // Ensure RequestId is available if Governor uses it
        .layer(GovernorLayer {
            config: governor_conf,
        });

    use axum::extract::connect_info::ConnectInfo;
    use std::net::SocketAddr;

    let addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

    // Send 2 requests, which should succeed
    let mut request1 = Request::builder().uri("/").body(Body::empty()).unwrap();
    request1.extensions_mut().insert(ConnectInfo(addr));
    let response1 = app.clone().oneshot(request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    let mut request2 = Request::builder().uri("/").body(Body::empty()).unwrap();
    request2.extensions_mut().insert(ConnectInfo(addr));
    let response2 = app.clone().oneshot(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::OK);

    // Send a 3rd request, which should be rate-limited
    let mut request3 = Request::builder().uri("/").body(Body::empty()).unwrap();
    request3.extensions_mut().insert(ConnectInfo(addr));
    let response3 = app.clone().oneshot(request3).await.unwrap();
    assert_eq!(response3.status(), StatusCode::TOO_MANY_REQUESTS);

    // Wait for the rate limit period to pass (plus a small buffer)
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    // Send another request, which should now succeed
    let mut request4 = Request::builder().uri("/").body(Body::empty()).unwrap();
    request4.extensions_mut().insert(ConnectInfo(addr));
    let response4 = app.oneshot(request4).await.unwrap();
    assert_eq!(response4.status(), StatusCode::OK);
}
