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
}