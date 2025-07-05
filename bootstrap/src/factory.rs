use std::sync::Arc;

use application::Container;
use contracts::ports::{DynObservability, DynUserRepo};
use infra_db_postgres::user_repo::PostgresUserRepository;
use infra_telemetry::{metrics::Metrics, config::TelemetryConfig};

use crate::config::Config;

/// 依賴工廠 - 負責組裝所有依賴
pub struct DependencyFactory;

impl DependencyFactory {
    /// 創建完整的依賴容器
    pub async fn create_container(
        config: &Config,
    ) -> Result<Container, Box<dyn std::error::Error>> {
        // 創建基礎設施適配器
        let user_repo = Self::create_user_repository(config).await?;
        let observability = Self::create_observability(config);

        // 組裝容器
        Ok(Container::new(user_repo, observability))
    }

    async fn create_user_repository(
        config: &Config,
    ) -> Result<DynUserRepo, Box<dyn std::error::Error>> {
        let repo = PostgresUserRepository::new(&config.database_url, config.db_max_conn).await?;
        Ok(Arc::new(repo))
    }

    fn create_observability(config: &Config) -> DynObservability {
        let telemetry_config = TelemetryConfig {
            otel_service_name: "rust-service-scaffold".to_string(),
            otel_exporter_otlp_endpoint: "http://localhost:4317".to_string(),
            prometheus_path: "/metrics".to_string(),
            log_level: "info".to_string(),
        };
        Arc::new(Metrics::new(&telemetry_config))
    }
}
