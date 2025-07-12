fn main() {
    println!("=== 測試 Telemetry 功能 ===");

    // 1. 測試 Prometheus Registry
    let registry = prometheus::Registry::new();
    let counter = prometheus::Counter::new("test_counter", "測試計數器").unwrap();
    registry.register(Box::new(counter.clone())).unwrap();
    counter.inc();

    let metrics = registry.gather();
    println!("✅ Prometheus Registry: 註冊了 {} 個指標", metrics.len());

    // 2. 測試基本日誌
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

    Registry::default()
        .with(EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("測試日誌訊息");
    println!("✅ 日誌系統: 已初始化並發送測試訊息");

    println!("✅ Telemetry 功能測試完成");
}
