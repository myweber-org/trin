use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub enable_tls: bool,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_yaml::from_str(&config_content)?;
        
        config.apply_environment_overrides();
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("DB_HOST") {
            self.database.host = host;
        }
        
        if let Ok(port) = env::var("DB_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                self.database.port = port_num;
            }
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
        
        if self.cache_ttl > 86400 {
            return Err("Cache TTL cannot exceed 24 hours".into());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
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
            self.database.database_name
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
    fn test_config_loading() {
        let config_yaml = r#"
database:
  host: localhost
  port: 5432
  username: postgres
  password: secret
  database_name: mydb
server:
  address: 0.0.0.0
  port: 8080
  enable_tls: false
  max_connections: 100
log_level: info
cache_ttl: 3600
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_yaml).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.log_level, "info");
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("DB_HOST", "prod-db.example.com");
        env::set_var("LOG_LEVEL", "debug");
        
        let config_yaml = r#"
database:
  host: localhost
  port: 5432
  username: postgres
  password: secret
  database_name: mydb
server:
  address: 0.0.0.0
  port: 8080
  enable_tls: false
  max_connections: 100
log_level: info
cache_ttl: 3600
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_yaml).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "prod-db.example.com");
        assert_eq!(config.log_level, "debug");
        
        env::remove_var("DB_HOST");
        env::remove_var("LOG_LEVEL");
    }
    
    #[test]
    fn test_validation() {
        let invalid_config = r#"
database:
  host: localhost
  port: 0
  username: postgres
  password: secret
  database_name: mydb
server:
  address: 0.0.0.0
  port: 8080
  enable_tls: false
  max_connections: 100
log_level: invalid_level
cache_ttl: 100000
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), invalid_config).unwrap();
        
        let result = AppConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }
}