// src/main.rs

use axum_logging_service::app::Application;
use axum_logging_service::config::Config;
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let config = Config::load().expect("Failed to load configuration.");

    let application = Application::build(config).await?;
    application.run_until_stopped().await?;

    tracing::info!("Telemetry provider shut down gracefully.");

    Ok(())
}
