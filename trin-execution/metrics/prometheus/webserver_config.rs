use serde::Deserialize;
use std::env;
use std::fs;
use std::net::SocketAddr;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: SocketAddr,
    pub workers: usize,
    pub log_level: String,
    pub static_dir: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            address: "127.0.0.1:8080".parse().unwrap(),
            workers: 4,
            log_level: "info".to_string(),
            static_dir: "./static".to_string(),
        }
    }
}

impl ServerConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());

        if let Ok(config_content) = fs::read_to_string(&config_path) {
            let mut config: ServerConfig = toml::from_str(&config_content)?;
            config.apply_env_overrides();
            Ok(config)
        } else {
            let mut config = ServerConfig::default();
            config.apply_env_overrides();
            Ok(config)
        }
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(addr) = env::var("SERVER_ADDRESS") {
            if let Ok(parsed_addr) = addr.parse() {
                self.address = parsed_addr;
            }
        }

        if let Ok(workers) = env::var("SERVER_WORKERS") {
            if let Ok(parsed_workers) = workers.parse() {
                self.workers = parsed_workers;
            }
        }

        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level.to_lowercase();
        }

        if let Ok(static_dir) = env::var("STATIC_DIR") {
            self.static_dir = static_dir;
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.workers == 0 {
            return Err("Worker count must be greater than zero".to_string());
        }

        if !std::path::Path::new(&self.static_dir).exists() {
            return Err(format!("Static directory '{}' does not exist", self.static_dir));
        }

        Ok(())
    }
}