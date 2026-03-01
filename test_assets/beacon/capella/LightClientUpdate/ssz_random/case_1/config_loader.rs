
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub features: HashMap<String, bool>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_json::from_str(&content)?;
        
        config.apply_environment_overrides();
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("APP_HOST") {
            self.server.host = host;
        }
        
        if let Ok(port) = env::var("APP_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.server.port = port_num;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database.url = db_url;
        }
    }
    
    pub fn is_feature_enabled(&self, feature_name: &str) -> bool {
        self.features.get(feature_name).copied().unwrap_or(false)
    }
}

pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_path = env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config.json".to_string());
    
    AppConfig::from_file(&config_path)
}