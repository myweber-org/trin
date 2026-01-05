use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

pub fn parse_config_file(file_path: &str) -> Result<Config, Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn validate_config(config: &Config) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be zero".to_string());
    }
    
    if config.database.max_connections == 0 {
        return Err("Database max connections must be greater than zero".to_string());
    }
    
    if config.database.timeout_seconds > 3600 {
        return Err("Database timeout cannot exceed one hour".to_string());
    }
    
    Ok(())
}

pub fn generate_default_config() -> Config {
    Config {
        server: ServerConfig {
            host: "localhost".to_string(),
            port: 8080,
            tls_enabled: false,
        },
        database: DatabaseConfig {
            url: "postgresql://localhost:5432/mydb".to_string(),
            max_connections: 10,
            timeout_seconds: 30,
        },
    }
}