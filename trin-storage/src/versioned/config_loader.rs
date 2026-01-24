use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub max_connections: u32,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let file_contents = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
        
        let mut config: Config = toml::from_str(&file_contents)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        
        if let Ok(host) = env::var("SERVER_HOST") {
            config.server_host = host;
        }
        
        if let Ok(port) = env::var("SERVER_PORT") {
            config.server_port = port.parse()
                .map_err(|e| format!("Invalid SERVER_PORT value: {}", e))?;
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }
        
        config.validate()?;
        Ok(config)
    }
    
    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        
        if self.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        Ok(())
    }
    
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}