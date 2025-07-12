use std::sync::{Arc, Mutex};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

// 簡單的測試收集器
#[derive(Debug, Clone)]
struct TestCollector {
    logs: Arc<Mutex<Vec<String>>>,
}

impl TestCollector {
    fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_logs(&self) -> Vec<String> {
        self.logs.lock().unwrap().clone()
    }
}

impl<S> tracing_subscriber::Layer<S> for TestCollector
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = TestVisitor::new();
        event.record(&mut visitor);
        self.logs.lock().unwrap().push(visitor.message);
    }
}

struct TestVisitor {
    message: String,
}

impl TestVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
}

impl tracing::field::Visit for TestVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
}

fn main() {
    // 1. 測試日誌功能
    println!("=== 測試 Telemetry 日誌功能 ===");
    
    let collector = TestCollector::new();
    let env_filter = EnvFilter::new("info");
    
    Registry::default()
        .with(env_filter)
        .with(collector.clone())
        .init();

    // 發送測試日誌
    info!("測試訊息 1");
    error!("測試錯誤訊息");
    info!("測試訊息 2");

    // 檢查收集到的日誌
    let logs = collector.get_logs();
    println!("收集到 {} 條日誌:", logs.len());
    for (i, log) in logs.iter().enumerate() {
        println!("  {}. {}", i + 1, log);
    }

    // 2. 測試 Prometheus Registry
    println!("\n=== 測試 Prometheus Registry ===");
    
    let registry = prometheus::Registry::new();
    
    // 創建測試指標
    let counter = prometheus::Counter::new("test_counter", "測試計數器").unwrap();
    registry.register(Box::new(counter.clone())).unwrap();
    
    // 增加計數
    counter.inc();
    counter.inc_by(5.0);
    
    // 獲取指標
    let metric_families = registry.gather();
    println!("註冊了 {} 個指標族:", metric_families.len());
    
    for family in &metric_families {
        println!("  指標: {}", family.get_name());
        println!("  說明: {}", family.get_help());
        for metric in family.get_metric() {
            println!("  值: {}", metric.get_counter().get_value());
        }
    }

    // 3. 測試 Panic Hook
    println!("\n=== 測試 Panic Hook ===");
    
    std::panic::set_hook(Box::new(|panic_info| {
        println!("捕獲到 Panic:");
        println!("  位置: {:?}", panic_info.location());
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            println!("  訊息: {}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            println!("  訊息: {}", s);
        }
    }));

    println!("Panic Hook 已設置");

    println!("\n✅ Telemetry 功能測試完成");
}