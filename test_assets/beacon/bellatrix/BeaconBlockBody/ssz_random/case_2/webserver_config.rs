use std::env;
use std::fs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub enable_tls: bool,
    pub log_level: String,
}

impl ServerConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let mut config: ServerConfig = toml::from_str(&config_str)?;
        
        config.apply_environment_overrides();
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("SERVER_HOST") {
            self.host = host;
        }
        
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.port = port_num;
            }
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level.to_uppercase();
        }
    }
    
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let config_content = r#"
            host = "127.0.0.1"
            port = 8080
            max_connections = 100
            enable_tls = false
            log_level = "INFO"
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_content).unwrap();
        
        let config = ServerConfig::from_file(temp_file.path().to_str().unwrap())
            .expect("Failed to parse config");
        
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.log_level, "INFO");
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("SERVER_HOST", "0.0.0.0");
        env::set_var("LOG_LEVEL", "debug");
        
        let config_content = r#"
            host = "localhost"
            port = 3000
            max_connections = 50
            enable_tls = true
            log_level = "WARN"
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_content).unwrap();
        
        let config = ServerConfig::from_file(temp_file.path().to_str().unwrap())
            .expect("Failed to parse config");
        
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.log_level, "DEBUG");
        
        env::remove_var("SERVER_HOST");
        env::remove_var("LOG_LEVEL");
    }
}