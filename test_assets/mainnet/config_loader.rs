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
        let config_str = fs::read_to_string(file_path)?;
        let config: AppConfig = toml::from_str(&config_str)?;
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

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(config) = Self::from_env() {
            return Ok(config);
        }

        if let Ok(config) = Self::from_file("config.toml") {
            return Ok(config);
        }

        Err("No configuration source found".into())
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".into());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".into());
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_validation() {
        let config = AppConfig {
            server_port: 8080,
            database_url: "postgres://localhost/test".to_string(),
            log_level: "info".to_string(),
            cache_ttl: 300,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_log_level() {
        let config = AppConfig {
            server_port: 8080,
            database_url: "postgres://localhost/test".to_string(),
            log_level: "invalid".to_string(),
            cache_ttl: 300,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_env_config() {
        env::set_var("SERVER_PORT", "8080");
        env::set_var("DATABASE_URL", "postgres://localhost/test");
        env::set_var("CACHE_TTL", "300");

        let config = AppConfig::from_env();
        assert!(config.is_ok());
    }
}use serde::Deserialize;
use std::env;
use std::fs;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid port number")]
    InvalidPort,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        let config_content = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;

        if let Ok(port) = env::var("SERVER_PORT") {
            config.server_port = port.parse().map_err(|_| ConfigError::InvalidPort)?;
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if config.server_port == 0 {
            return Err(ConfigError::InvalidPort);
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server_port > 65535 {
            return Err(ConfigError::InvalidPort);
        }

        if self.database_url.is_empty() {
            return Err(ConfigError::MissingEnvVar("DATABASE_URL".to_string()));
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
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_port = 8080
            database_url = "postgres://localhost/db"
            log_level = "info"
            cache_ttl = 300
        "#;
        std::fs::write(temp_file.path(), config_content).unwrap();

        std::env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        
        let config = AppConfig::load();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgres://localhost/db");
    }

    #[test]
    fn test_env_override() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_port = 8080
            database_url = "original"
            log_level = "info"
            cache_ttl = 300
        "#;
        std::fs::write(temp_file.path(), config_content).unwrap();

        std::env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        std::env::set_var("DATABASE_URL", "overridden");
        
        let config = AppConfig::load().unwrap();
        assert_eq!(config.database_url, "overridden");
    }
}