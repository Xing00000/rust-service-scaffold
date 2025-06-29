// src/config.rs

use contracts::HasPort;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, Debug, Clone)]
pub struct HttpHeader {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub value: String,
}

#[derive(Deserialize, Validate, Debug, Clone)]
pub struct Config {
    #[validate(range(min = 1024, max = 65535))]
    pub port: u16,

    #[validate(length(min = 1))]
    pub log_level: String,

    #[validate(url)]
    pub otel_exporter_otlp_endpoint: String,

    #[validate(length(min = 1))]
    pub otel_service_name: String,

    // <-- 新增: 限流器每秒允許的請求數量
    #[validate(range(min = 1))]
    pub rate_limit_per_second: u64,

    // <-- 新增: 限流器允許的突發請求數量
    #[validate(range(min = 1))]
    pub rate_limit_burst_size: u32,

    pub http_headers: Option<Vec<HttpHeader>>,

    #[validate(length(min = 1))]
    pub database_url: String,

    #[validate(range(min = 1))]
    pub db_max_conn: u32,
}

impl Config {
    /// 從文件和環境變量加載配置，並進行驗證。
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config: Config = Figment::new()
            .merge(Toml::file("config/default.toml"))
            .merge(Env::prefixed("APP_"))
            .extract()?; // Figment 的錯誤會被自動轉換為 Box<dyn Error>

        // 驗證配置，如果失敗則返回一個 boxed error
        config.validate()?; // validator 的錯誤也會被自動轉換

        Ok(config)
    }
}

// app/config.rs
impl HasPort for Config {
    fn port(&self) -> u16 {
        self.port
    }
}
