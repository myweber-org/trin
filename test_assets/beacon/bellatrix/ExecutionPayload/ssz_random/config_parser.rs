use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub pool_timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_file_size_mb: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/appdb".to_string(),
                max_connections: 10,
                pool_timeout_seconds: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "logs/app.log".to_string(),
                max_file_size_mb: 100,
            },
        }
    }
}

pub fn load_config(config_path: &str) -> Result<AppConfig, String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    validate_config(&config)?;
    Ok(config)
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }
    
    if config.database.max_connections == 0 {
        return Err("Database max connections cannot be 0".to_string());
    }
    
    if config.logging.max_file_size_mb == 0 {
        return Err("Log file max size cannot be 0".to_string());
    }
    
    Ok(())
}

pub fn generate_default_config(config_path: &str) -> Result<(), String> {
    let default_config = AppConfig::default();
    let toml_content = toml::to_string_pretty(&default_config)
        .map_err(|e| format!("Failed to serialize default config: {}", e))?;
    
    fs::write(config_path, toml_content)
        .map_err(|e| format!("Failed to write default config: {}", e))?;
    
    Ok(())
}

pub fn merge_configs(base: AppConfig, overrides: HashMap<String, String>) -> AppConfig {
    let mut merged = base;
    
    for (key, value) in overrides {
        match key.as_str() {
            "server.host" => merged.server.host = value,
            "server.port" => if let Ok(port) = value.parse() { merged.server.port = port },
            "server.timeout_seconds" => if let Ok(timeout) = value.parse() { merged.server.timeout_seconds = timeout },
            "database.url" => merged.database.url = value,
            "database.max_connections" => if let Ok(conns) = value.parse() { merged.database.max_connections = conns },
            "database.pool_timeout_seconds" => if let Ok(timeout) = value.parse() { merged.database.pool_timeout_seconds = timeout },
            "logging.level" => merged.logging.level = value,
            "logging.file_path" => merged.logging.file_path = value,
            "logging.max_file_size_mb" => if let Ok(size) = value.parse() { merged.logging.max_file_size_mb = size },
            _ => (),
        }
    }
    
    merged
}