use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub pool_timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_file_size_mb: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                max_connections: 10,
                pool_timeout_seconds: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                max_file_size_mb: 100,
            },
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(&path)
            .map_err(|e| ConfigError::FileRead(path.as_ref().to_path_buf(), e))?;
        
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::Parse(e))?;
        
        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(&path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: Failed to load config from {:?}: {}. Using defaults.", path.as_ref(), e);
                AppConfig::default()
            }
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::Validation("Server port cannot be 0".to_string()));
        }
        
        if self.database.max_connections == 0 {
            return Err(ConfigError::Validation("Database max connections cannot be 0".to_string()));
        }
        
        if self.logging.max_file_size_mb == 0 {
            return Err(ConfigError::Validation("Log file max size cannot be 0".to_string()));
        }
        
        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::Serialize(e))?;
        
        fs::write(&path, content)
            .map_err(|e| ConfigError::FileWrite(path.as_ref().to_path_buf(), e))?;
        
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileRead(std::path::PathBuf, std::io::Error),
    FileWrite(std::path::PathBuf, std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
    Validation(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileRead(path, err) => write!(f, "Failed to read config file {:?}: {}", path, err),
            ConfigError::FileWrite(path, err) => write!(f, "Failed to write config file {:?}: {}", path, err),
            ConfigError::Parse(err) => write!(f, "Failed to parse config: {}", err),
            ConfigError::Serialize(err) => write!(f, "Failed to serialize config: {}", err),
            ConfigError::Validation(msg) => write!(f, "Config validation failed: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}