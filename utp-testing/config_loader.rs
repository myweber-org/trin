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
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;

        let mut config: AppConfig = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        if let Ok(port) = env::var("SERVER_PORT") {
            config.server_port = port.parse()
                .map_err(|e| format!("Invalid SERVER_PORT value: {}", e))?;
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if config.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }

        if config.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.server_port < 1024 || self.server_port > 65535 {
            errors.push(format!("Server port {} is out of valid range (1024-65535)", self.server_port));
        }

        if !self.database_url.starts_with("postgres://") && !self.database_url.starts_with("mysql://") {
            errors.push("Database URL must start with postgres:// or mysql://".to_string());
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            errors.push(format!("Invalid log level: {}. Must be one of: {:?}", self.log_level, valid_log_levels));
        }

        if self.cache_ttl > 86400 {
            errors.push("Cache TTL cannot exceed 24 hours (86400 seconds)".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
use serde::Deserialize;
use std::env;
use std::fs;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub server_port: u16,
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
    
    #[error("Invalid configuration value: {0}")]
    ValidationError(String),
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());
        
        let config_content = fs::read_to_string(&config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;
        
        config.apply_environment_overrides()?;
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) -> Result<(), ConfigError> {
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(port) = env::var("SERVER_PORT") {
            self.server_port = port.parse()
                .map_err(|_| ConfigError::ValidationError(
                    format!("Invalid port number: {}", port)
                ))?;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
        
        Ok(())
    }
    
    fn validate(&self) -> Result<(), ConfigError> {
        if self.database_url.is_empty() {
            return Err(ConfigError::ValidationError(
                "Database URL cannot be empty".to_string()
            ));
        }
        
        if self.server_port == 0 {
            return Err(ConfigError::ValidationError(
                "Server port must be greater than 0".to_string()
            ));
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(ConfigError::ValidationError(
                format!("Invalid log level: {}", self.log_level)
            ));
        }
        
        Ok(())
    }
    
    pub fn get_database_connection_string(&self) -> String {
        format!("{}?connection_limit=10", self.database_url)
    }
}