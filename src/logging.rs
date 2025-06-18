use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt};

pub fn init_subscriber() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_logging_service=debug,tower_http=debug".into()),
        )
        .with(fmt::layer().json()) // Ensure JSON output
        .init();
}
