// src/main.rs

use axum_logging_service::app::Application; // ✅ 修正: 將本 crate 作為庫導入
use axum_logging_service::config::Config; // ✅ 修正: 將本 crate 作為庫導入
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // 這個臨時日誌記錄器仍然有用，用於捕獲配置加載錯誤
    let subscriber = tracing_subscriber::fmt().with_env_filter("info").finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config = Config::load().expect("Failed to load configuration.");

    // Application 和 Config 現在來自庫
    let application = Application::build(config).await?;
    application.run_until_stopped().await?;

    Ok(())
}
