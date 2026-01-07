use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server_address: String,
    pub port: u16,
    pub database_url: String,
    pub enable_logging: bool,
    pub log_level: String,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let config_content = fs::read_to_string(&config_path)?;
        let mut config: Config = toml::from_str(&config_content)?;
        
        if let Ok(server_address) = env::var("SERVER_ADDRESS") {
            config.server_address = server_address;
        }
        
        if let Ok(port) = env::var("PORT") {
            config.port = port.parse()?;
        }
        
        if let Ok(database_url) = env::var("DATABASE_URL") {
            config.database_url = database_url;
        }
        
        if let Ok(enable_logging) = env::var("ENABLE_LOGGING") {
            config.enable_logging = enable_logging.parse().unwrap_or(false);
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level;
        }
        
        config.validate()?;
        Ok(config)
    }
    
    fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }
        
        if self.server_address.is_empty() {
            return Err("Server address cannot be empty".to_string());
        }
        
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_address = "127.0.0.1"
            port = 8080
            database_url = "postgres://localhost/mydb"
            enable_logging = true
            log_level = "info"
        "#;
        
        temp_file.write_all(config_content.as_bytes()).unwrap();
        
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        
        let config = Config::load().unwrap();
        assert_eq!(config.server_address, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.database_url, "postgres://localhost/mydb");
        assert_eq!(config.enable_logging, true);
        assert_eq!(config.log_level, "info");
        
        env::remove_var("CONFIG_PATH");
    }
    
    #[test]
    fn test_environment_variable_override() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            server_address = "127.0.0.1"
            port = 8080
            database_url = "postgres://localhost/mydb"
            enable_logging = false
            log_level = "warn"
        "#;
        
        temp_file.write_all(config_content.as_bytes()).unwrap();
        
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        env::set_var("PORT", "9090");
        env::set_var("ENABLE_LOGGING", "true");
        
        let config = Config::load().unwrap();
        assert_eq!(config.port, 9090);
        assert_eq!(config.enable_logging, true);
        
        env::remove_var("CONFIG_PATH");
        env::remove_var("PORT");
        env::remove_var("ENABLE_LOGGING");
    }
    
    #[test]
    fn test_config_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let invalid_config = r#"
            server_address = ""
            port = 0
            database_url = ""
            enable_logging = true
            log_level = "invalid"
        "#;
        
        temp_file.write_all(invalid_config.as_bytes()).unwrap();
        
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        
        let result = Config::load();
        assert!(result.is_err());
        
        env::remove_var("CONFIG_PATH");
    }
}