use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    #[serde(default = "default_retry_attempts")]
    pub max_retries: u8,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    #[serde(default = "default_log_format")]
    pub format: String,
}

fn default_timeout() -> u64 {
    30
}

fn default_retry_attempts() -> u8 {
    3
}

fn default_log_format() -> String {
    "json".to_string()
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server.port == 0 {
            return Err("Port cannot be zero".into());
        }
        if self.database.pool_size == 0 {
            return Err("Database pool size cannot be zero".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let toml_content = r#"
            [server]
            host = "localhost"
            port = 8080

            [database]
            url = "postgresql://localhost/mydb"
            pool_size = 10

            [logging]
            level = "info"
        "#;

        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(file.path()).unwrap();
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.timeout, 30);
        assert_eq!(config.database.max_retries, 3);
        assert_eq!(config.logging.format, "json");
    }

    #[test]
    fn test_config_validation() {
        let invalid_toml = r#"
            [server]
            host = "localhost"
            port = 0

            [database]
            url = "postgresql://localhost/mydb"
            pool_size = 10

            [logging]
            level = "info"
        "#;

        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), invalid_toml).unwrap();

        let result = AppConfig::from_file(file.path());
        assert!(result.is_err());
    }
}