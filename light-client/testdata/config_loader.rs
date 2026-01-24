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
        let contents = fs::read_to_string(file_path)?;
        let config: AppConfig = toml::from_str(&contents)?;
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

        let config_path = env::var("CONFIG_FILE")
            .unwrap_or_else(|_| "config.toml".to_string());
        
        Self::from_file(&config_path)
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