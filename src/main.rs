// src/main.rs

use axum_logging_service::app::Application;
use axum_logging_service::config::Config;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let subscriber = tracing_subscriber::fmt().with_env_filter("info").finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config = Config::load().expect("Failed to load configuration.");

    let application = Application::build(config).await?;
    application.run_until_stopped().await?;

    // ✅ [關鍵新增] 服務停止後，執行清理操作
    // 這會確保所有緩存的追踪數據都被發送到 OTLP 收集器
    opentelemetry::global::shutdown_tracer_provider();
    tracing::info!("Telemetry provider shut down gracefully.");

    Ok(())
}
