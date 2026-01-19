
use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub max_connections: u32,
}

impl AppConfig {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(file_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;
        
        config.apply_environment_overrides();
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("SERVER_HOST") {
            self.server_host = host;
        }
        
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.server_port = port_num;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level.to_uppercase();
        }
        
        if let Ok(max_conn) = env::var("MAX_CONNECTIONS") {
            if let Ok(max_conn_num) = max_conn.parse::<u32>() {
                self.max_connections = max_conn_num;
            }
        }
    }
    
    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        
        if self.max_connections == 0 {
            return Err("Max connections must be greater than zero".to_string());
        }
        
        let valid_log_levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        if !self.database_url.starts_with("postgres://") && 
           !self.database_url.starts_with("mysql://") {
            return Err("Database URL must start with postgres:// or mysql://".to_string());
        }
        
        Ok(())
    }
    
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_loading() {
        let config_content = r#"
            server_host = "localhost"
            server_port = 8080
            database_url = "postgres://user:pass@localhost/db"
            log_level = "INFO"
            max_connections = 100
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), config_content).unwrap();
        
        let config = AppConfig::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.server_host, "localhost");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "INFO");
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.server_address(), "localhost:8080");
    }
    
    #[test]
    fn test_environment_overrides() {
        std::env::set_var("SERVER_PORT", "9090");
        std::env::set_var("LOG_LEVEL", "debug");
        
        let config_content = r#"
            server_host = "localhost"
            server_port = 8080
            database_url = "postgres://user:pass@localhost/db"
            log_level = "INFO"
            max_connections = 100
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), config_content).unwrap();
        
        let config = AppConfig::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.log_level, "DEBUG");
        
        std::env::remove_var("SERVER_PORT");
        std::env::remove_var("LOG_LEVEL");
    }
    
    #[test]
    fn test_validation_failure() {
        let config_content = r#"
            server_host = "localhost"
            server_port = 0
            database_url = "invalid://url"
            log_level = "INVALID"
            max_connections = 0
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), config_content).unwrap();
        
        let result = AppConfig::from_file(file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}