use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;
        
        if let Ok(port) = env::var("APP_PORT") {
            if let Ok(parsed_port) = port.parse::<u16>() {
                config.server_port = parsed_port;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level.to_lowercase();
        }
        
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero");
        }
        
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty");
        }
        
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err("Invalid log level specified");
        }
        
        Ok(())
    }
}