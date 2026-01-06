use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    pub fn to_file(&self, path: &str) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        
        fs::write(path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::ValidationError("Port cannot be zero".to_string()));
        }
        
        if self.database.max_connections == 0 {
            return Err(ConfigError::ValidationError("Max connections must be greater than zero".to_string()));
        }

        Ok(())
    }

    pub fn get_env_overrides(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        env_vars.insert("SERVER_HOST".to_string(), self.server.host.clone());
        env_vars.insert("SERVER_PORT".to_string(), self.server.port.to_string());
        env_vars.insert("DATABASE_URL".to_string(), self.database.url.clone());
        env_vars.insert("LOG_LEVEL".to_string(), self.logging.level.clone());
        env_vars
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn load_config_with_fallback(paths: &[&str]) -> Result<AppConfig, ConfigError> {
    for path in paths {
        match AppConfig::from_file(path) {
            Ok(config) => {
                if let Err(e) = config.validate() {
                    eprintln!("Warning: Config validation failed for {}: {}", path, e);
                }
                return Ok(config);
            }
            Err(e) => {
                eprintln!("Failed to load config from {}: {}", path, e);
                continue;
            }
        }
    }
    
    Err(ConfigError::IoError("No valid configuration file found".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_serialization() {
        let config = AppConfig {
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 8080,
                enable_ssl: false,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/mydb".to_string(),
                max_connections: 10,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("/var/log/app.log".to_string()),
                enable_console: true,
            },
        };

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        config.to_file(path).unwrap();
        let loaded = AppConfig::from_file(path).unwrap();
        
        assert_eq!(config.server.host, loaded.server.host);
        assert_eq!(config.server.port, loaded.server.port);
        assert_eq!(config.database.url, loaded.database.url);
    }

    #[test]
    fn test_config_validation() {
        let invalid_config = AppConfig {
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 0,
                enable_ssl: false,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/mydb".to_string(),
                max_connections: 0,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                enable_console: true,
            },
        };

        assert!(invalid_config.validate().is_err());
    }
}