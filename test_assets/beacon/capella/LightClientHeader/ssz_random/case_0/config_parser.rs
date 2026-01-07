
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_yaml::from_str(&content)?;
        
        config.resolve_environment_variables();
        config.validate()?;
        
        Ok(config)
    }
    
    fn resolve_environment_variables(&mut self) {
        if let Ok(host) = env::var("DB_HOST") {
            self.database.host = host;
        }
        
        if let Ok(port) = env::var("DB_PORT") {
            if let Ok(port_num) = port.parse() {
                self.database.port = port_num;
            }
        }
        
        if let Ok(username) = env::var("DB_USERNAME") {
            self.database.username = username;
        }
        
        if let Ok(password) = env::var("DB_PASSWORD") {
            self.database.password = password;
        }
        
        if let Ok(database) = env::var("DB_NAME") {
            self.database.database = database;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
    }
    
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.database.port == 0 {
            return Err("Database port cannot be zero".into());
        }
        
        if self.server.port == 0 {
            return Err("Server port cannot be zero".into());
        }
        
        if self.server.workers == 0 {
            return Err("Number of workers cannot be zero".into());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.to_lowercase().as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level).into());
        }
        
        Ok(())
    }
    
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }
    
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.address, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let yaml_content = r#"
database:
  host: localhost
  port: 5432
  username: postgres
  password: secret
  database: myapp

server:
  address: 0.0.0.0
  port: 8080
  workers: 4

log_level: info
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), yaml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.database_url(), "postgres://postgres:secret@localhost:5432/myapp");
        assert_eq!(config.server_address(), "0.0.0.0:8080");
    }
    
    #[test]
    fn test_environment_variable_override() {
        env::set_var("DB_HOST", "remote-host");
        env::set_var("LOG_LEVEL", "debug");
        
        let yaml_content = r#"
database:
  host: localhost
  port: 5432
  username: postgres
  password: secret
  database: myapp

server:
  address: 0.0.0.0
  port: 8080
  workers: 4

log_level: info
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), yaml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "remote-host");
        assert_eq!(config.log_level, "debug");
        
        env::remove_var("DB_HOST");
        env::remove_var("LOG_LEVEL");
    }
    
    #[test]
    fn test_validation() {
        let invalid_yaml = r#"
database:
  host: localhost
  port: 0
  username: postgres
  password: secret
  database: myapp

server:
  address: 0.0.0.0
  port: 8080
  workers: 4

log_level: info
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), invalid_yaml).unwrap();
        
        let result = AppConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }
}