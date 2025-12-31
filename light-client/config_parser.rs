use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_size: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/appdb"),
            log_level: String::from("info"),
            cache_size: 100,
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err(String::from("Server port cannot be zero"));
        }

        if self.database_url.is_empty() {
            return Err(String::from("Database URL cannot be empty"));
        }

        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!(
                "Invalid log level '{}'. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }

        if self.cache_size > 10000 {
            return Err(String::from("Cache size cannot exceed 10000"));
        }

        Ok(())
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

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
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        assert!(config.to_file(temp_file.path()).is_ok());
        let loaded = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.server_port, loaded.server_port);
        assert_eq!(config.database_url, loaded.database_url);
    }
}