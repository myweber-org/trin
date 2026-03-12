use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(file_path)?;
        let config: AppConfig = toml::from_str(&config_content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let config = AppConfig {
            server_port: env::var("SERVER_PORT")?.parse()?,
            database_url: env::var("DATABASE_URL")?,
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            cache_ttl: env::var("CACHE_TTL")?.parse()?,
        };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".into());
        }
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".into());
        }
        if self.cache_ttl > 86400 {
            return Err("Cache TTL cannot exceed 24 hours".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_from_file() {
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost/db"
            log_level = "debug"
            cache_ttl = 3600
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), config_content).unwrap();

        let config = AppConfig::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_ttl, 3600);
    }

    #[test]
    fn test_config_from_env() {
        env::set_var("SERVER_PORT", "3000");
        env::set_var("DATABASE_URL", "sqlite://data.db");
        env::set_var("CACHE_TTL", "1800");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.database_url, "sqlite://data.db");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 1800);
    }

    #[test]
    fn test_config_validation() {
        let invalid_config = AppConfig {
            server_port: 0,
            database_url: "".to_string(),
            log_level: "info".to_string(),
            cache_ttl: 100000,
        };

        assert!(invalid_config.validate().is_err());
    }
}