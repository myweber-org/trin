use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub log_level: String,
    pub static_dir: Option<String>,
}

impl ServerConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: ServerConfig = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 100,
            log_level: "info".to_string(),
            static_dir: Some("./static".to_string()),
        }
    }
}