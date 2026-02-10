use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub enable_cache: bool,
    pub cache_ttl: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_host: String::from("127.0.0.1"),
            server_port: 8080,
            database_url: String::from("postgresql://localhost/app_db"),
            log_level: String::from("info"),
            enable_cache: true,
            cache_ttl: 300,
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
        if self.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        
        if self.cache_ttl > 86400 {
            return Err("Cache TTL cannot exceed 24 hours".to_string());
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
        assert_eq!(config.server_host, "127.0.0.1");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.enable_cache);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server_port = 0;
        assert!(config.validate().is_err());
        
        config.server_port = 8080;
        config.cache_ttl = 100000;
        assert!(config.validate().is_err());
        
        config.cache_ttl = 300;
        config.log_level = String::from("invalid");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = AppConfig::default();
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();
        
        config.to_file(path)?;
        let loaded_config = AppConfig::from_file(path)?;
        
        assert_eq!(config.server_host, loaded_config.server_host);
        assert_eq!(config.server_port, loaded_config.server_port);
        Ok(())
    }
}