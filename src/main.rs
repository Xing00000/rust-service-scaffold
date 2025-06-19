use axum_logging_service::app::Application; // Import the Application struct
use std::error::Error; // For Box<dyn Error>

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let application = Application::build().await?;
    application.run_until_stopped().await?;
    Ok(())
}
