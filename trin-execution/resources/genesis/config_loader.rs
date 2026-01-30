use serde::Deserialize;
use std::env;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()?;

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost:5432/appdb".to_string());

        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string())
            .to_lowercase();

        let cache_ttl = env::var("CACHE_TTL")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()?;

        let config = AppConfig {
            server_port,
            database_url,
            log_level,
            cache_ttl,
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".into());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".into());
        }

        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level).into());
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

    #[test]
    fn test_default_config() {
        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_URL");
        env::remove_var("LOG_LEVEL");
        env::remove_var("CACHE_TTL");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgres://localhost:5432/appdb");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_custom_config() {
        env::set_var("SERVER_PORT", "9090");
        env::set_var("DATABASE_URL", "postgres://prod:5432/mydb");
        env::set_var("LOG_LEVEL", "DEBUG");
        env::set_var("CACHE_TTL", "600");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.database_url, "postgres://prod:5432/mydb");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_ttl, 600);

        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_URL");
        env::remove_var("LOG_LEVEL");
        env::remove_var("CACHE_TTL");
    }

    #[test]
    fn test_invalid_port() {
        env::set_var("SERVER_PORT", "0");
        let result = AppConfig::from_env();
        assert!(result.is_err());
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_invalid_log_level() {
        env::set_var("LOG_LEVEL", "INVALID");
        let result = AppConfig::from_env();
        assert!(result.is_err());
        env::remove_var("LOG_LEVEL");
    }
}