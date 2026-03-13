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
    pub log_level: String,
    pub enable_cache: bool,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_content)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server.port == 0 {
            return Err("Server port cannot be zero".into());
        }
        
        if self.database.port == 0 {
            return Err("Database port cannot be zero".into());
        }
        
        if self.server.max_connections == 0 {
            return Err("Max connections must be greater than zero".into());
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
    fn test_valid_config_parsing() {
        let config_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"
            database_name = "mydb"
            
            [server]
            address = "0.0.0.0"
            port = 8080
            max_connections = 100
            timeout_seconds = 30
            
            log_level = "info"
            enable_cache = true
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), config_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.enable_cache, true);
        assert_eq!(config.database_url(), "postgres://admin:secret@localhost:5432/mydb");
        assert_eq!(config.server_address(), "0.0.0.0:8080");
    }
    
    #[test]
    fn test_invalid_config_validation() {
        let config_content = r#"
            [database]
            host = "localhost"
            port = 0
            username = "admin"
            password = "secret"
            database_name = "mydb"
            
            [server]
            address = "0.0.0.0"
            port = 8080
            max_connections = 100
            timeout_seconds = 30
            
            log_level = "invalid_level"
            enable_cache = true
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), config_content).unwrap();
        
        let result = AppConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }
}