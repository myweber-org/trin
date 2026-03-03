use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub enable_logging: bool,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_address: String::from("127.0.0.1"),
            port: 8080,
            max_connections: 100,
            enable_logging: true,
            log_level: String::from("info"),
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err(String::from("Port cannot be zero"));
        }
        if self.max_connections == 0 {
            return Err(String::from("Max connections must be greater than zero"));
        }
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        Ok(())
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server_address, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert!(config.enable_logging);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        assert!(config.validate().is_ok());

        config.port = 0;
        assert!(config.validate().is_err());

        config.port = 8080;
        config.max_connections = 0;
        assert!(config.validate().is_err());

        config.max_connections = 100;
        config.log_level = String::from("invalid");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_file_operations() -> Result<(), Box<dyn std::error::Error>> {
        let config = AppConfig::default();
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        config.to_file(path)?;
        let loaded_config = AppConfig::from_file(path)?;
        
        assert_eq!(config.server_address, loaded_config.server_address);
        assert_eq!(config.port, loaded_config.port);
        assert_eq!(config.max_connections, loaded_config.max_connections);
        assert_eq!(config.enable_logging, loaded_config.enable_logging);
        assert_eq!(config.log_level, loaded_config.log_level);
        
        Ok(())
    }
}