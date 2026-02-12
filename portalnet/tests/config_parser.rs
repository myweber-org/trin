use std::fs;
use std::collections::HashMap;
use serde::Deserialize;
use toml;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::ValidationError("Port cannot be zero".to_string()));
        }
        
        if self.database.max_connections == 0 {
            return Err(ConfigError::ValidationError("Max connections must be greater than zero".to_string()));
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::ValidationError(format!(
                "Invalid log level: {}. Valid options are: {:?}",
                self.logging.level, valid_log_levels
            )));
        }

        Ok(())
    }

    pub fn to_env_map(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        env_vars.insert("SERVER_HOST".to_string(), self.server.host.clone());
        env_vars.insert("SERVER_PORT".to_string(), self.server.port.to_string());
        env_vars.insert("DB_URL".to_string(), self.database.url.clone());
        env_vars.insert("LOG_LEVEL".to_string(), self.logging.level.clone());
        env_vars
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_config_parsing() {
        let toml_content = r#"
            [server]
            host = "localhost"
            port = 8080
            enable_ssl = false

            [database]
            url = "postgresql://localhost/mydb"
            max_connections = 10
            timeout_seconds = 30

            [logging]
            level = "info"
            file_path = "/var/log/app.log"
            enable_console = true
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path().to_str().unwrap());
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_invalid_port() {
        let toml_content = r#"
            [server]
            host = "localhost"
            port = 0
            enable_ssl = false

            [database]
            url = "postgresql://localhost/mydb"
            max_connections = 10
            timeout_seconds = 30

            [logging]
            level = "info"
            enable_console = true
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path().to_str().unwrap());
        assert!(config.is_err());
        
        if let Err(ConfigError::ValidationError(msg)) = config {
            assert!(msg.contains("Port cannot be zero"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_env_map_generation() {
        let config = AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                enable_ssl: false,
            },
            database: DatabaseConfig {
                url: "mysql://localhost/test".to_string(),
                max_connections: 5,
                timeout_seconds: 10,
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                file_path: None,
                enable_console: true,
            },
        };

        let env_map = config.to_env_map();
        assert_eq!(env_map.get("SERVER_HOST"), Some(&"127.0.0.1".to_string()));
        assert_eq!(env_map.get("SERVER_PORT"), Some(&"3000".to_string()));
        assert_eq!(env_map.get("DB_URL"), Some(&"mysql://localhost/test".to_string()));
        assert_eq!(env_map.get("LOG_LEVEL"), Some(&"debug".to_string()));
    }
}