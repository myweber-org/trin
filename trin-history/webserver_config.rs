use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout_secs: u64,
    pub static_files_path: PathBuf,
    pub enable_compression: bool,
    pub enable_logging: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".parse().unwrap(),
            port: 8080,
            max_connections: 100,
            request_timeout_secs: 30,
            static_files_path: PathBuf::from("./static"),
            enable_compression: true,
            enable_logging: true,
        }
    }
}

impl ServerConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }
        if self.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }
        if self.request_timeout_secs == 0 {
            return Err("Request timeout must be greater than 0".to_string());
        }
        Ok(())
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ServerConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let mut config = ServerConfig::default();
        config.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = ServerConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("port = 8080"));
        assert!(serialized.contains("host = \"127.0.0.1\""));
    }
}