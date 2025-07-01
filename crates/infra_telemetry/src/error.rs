use thiserror::Error;

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("Telemetry initialization failed: {0}")]
    TelemetryInit(String),
    #[error("Telemetry (Metrics) initialization failed: {0}")]
    MetricsInit(String),
}
