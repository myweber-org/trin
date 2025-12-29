use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub enable_tls: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: String::from("127.0.0.1"),
            port: 8080,
            max_connections: 100,
            enable_tls: false,
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: ServerConfig = toml::from_str(&config_str)?;
    
    validate_config(&config)?;
    Ok(config)
}

pub fn load_config_with_defaults<P: AsRef<Path>>(path: P) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    match load_config(path) {
        Ok(config) => Ok(config),
        Err(_) => {
            println!("Using default configuration");
            Ok(ServerConfig::default())
        }
    }
}

fn validate_config(config: &ServerConfig) -> Result<(), String> {
    if config.port == 0 {
        return Err("Port cannot be zero".to_string());
    }
    
    if config.max_connections == 0 {
        return Err("Max connections must be greater than zero".to_string());
    }
    
    if config.host.is_empty() {
        return Err("Host cannot be empty".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert!(!config.enable_tls);
    }
    
    #[test]
    fn test_valid_config_loading() {
        let toml_content = r#"
            host = "192.168.1.100"
            port = 9000
            max_connections = 500
            enable_tls = true
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = load_config(temp_file.path()).unwrap();
        assert_eq!(config.host, "192.168.1.100");
        assert_eq!(config.port, 9000);
        assert_eq!(config.max_connections, 500);
        assert!(config.enable_tls);
    }
    
    #[test]
    fn test_invalid_config_validation() {
        let invalid_config = ServerConfig {
            host: String::new(),
            port: 0,
            max_connections: 0,
            enable_tls: false,
        };
        
        let result = validate_config(&invalid_config);
        assert!(result.is_err());
    }
}