use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub enable_https: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "".to_string(),
                database_name: "app_db".to_string(),
                max_connections: 10,
            },
            server: ServerConfig {
                address: "127.0.0.1".to_string(),
                port: 8080,
                enable_https: false,
                cert_path: None,
                key_path: None,
            },
            log_level: "info".to_string(),
            cache_ttl: 300,
        }
    }
}

pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError::IoError(error)
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let config_path = path.as_ref();
        
        if !config_path.exists() {
            return Err(ConfigError::FileNotFound(
                config_path.to_string_lossy().to_string()
            ));
        }

        let config_content = fs::read_to_string(config_path)?;
        
        let config: AppConfig = toml::from_str(&config_content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {:?}, using defaults", e);
                AppConfig::default()
            }
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.database.host.is_empty() {
            return Err(ConfigError::ValidationError(
                "Database host cannot be empty".to_string()
            ));
        }

        if self.database.port == 0 {
            return Err(ConfigError::ValidationError(
                "Database port cannot be zero".to_string()
            ));
        }

        if self.server.port == 0 {
            return Err(ConfigError::ValidationError(
                "Server port cannot be zero".to_string()
            ));
        }

        if self.server.enable_https {
            if self.server.cert_path.is_none() || self.server.key_path.is_none() {
                return Err(ConfigError::ValidationError(
                    "HTTPS requires both certificate and key paths".to_string()
                ));
            }
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(ConfigError::ValidationError(
                format!("Invalid log level: {}", self.log_level)
            ));
        }

        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        fs::write(path, toml_string)?;
        Ok(())
    }

    pub fn get_database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database_name
        )
    }

    pub fn get_server_url(&self) -> String {
        let protocol = if self.server.enable_https { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.server.address, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_validation() {
        let mut config = AppConfig::default();
        config.database.host = String::new();
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_save_and_load() {
        let mut config = AppConfig::default();
        config.log_level = "debug".to_string();
        
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        config.save_to_file(path).unwrap();
        let loaded_config = AppConfig::load_from_file(path).unwrap();
        
        assert_eq!(loaded_config.log_level, "debug");
    }

    #[test]
    fn test_get_urls() {
        let config = AppConfig::default();
        let db_url = config.get_database_url();
        let server_url = config.get_server_url();
        
        assert!(db_url.contains("postgres://"));
        assert!(server_url.contains("http://"));
    }
}