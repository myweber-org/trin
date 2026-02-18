use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub debug_mode: bool,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_content)?;
        
        if config.server.port == 0 {
            return Err("Server port cannot be zero".into());
        }
        
        if config.database.host.is_empty() {
            return Err("Database host cannot be empty".into());
        }
        
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.server.port > 65535 {
            errors.push("Server port must be between 1 and 65535".to_string());
        }
        
        if self.server.max_connections == 0 {
            errors.push("Max connections must be greater than zero".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            errors.push(format!("Log level must be one of: {}", valid_log_levels.join(", ")));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_config_parsing() {
        let config_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
            database_name = "app_db"
            
            [server]
            address = "0.0.0.0"
            port = 8080
            max_connections = 100
            timeout_seconds = 30
            
            debug_mode = true
            log_level = "info"
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), config_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path());
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.log_level, "info");
    }
    
    #[test]
    fn test_invalid_port_validation() {
        let config_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
            database_name = "app_db"
            
            [server]
            address = "0.0.0.0"
            port = 0
            max_connections = 100
            timeout_seconds = 30
            
            debug_mode = false
            log_level = "warn"
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), config_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path());
        assert!(config.is_err());
    }
}