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
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub debug_mode: bool,
    pub log_level: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            username: env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string()),
            database_name: env::var("DB_NAME").unwrap_or_else(|_| "app_db".to_string()),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            address: env::var("SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            max_connections: env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            debug_mode: env::var("DEBUG_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn from_env() -> Self {
        AppConfig::default()
    }

    pub fn merge_with_env(&mut self) {
        if let Ok(host) = env::var("DB_HOST") {
            self.database.host = host;
        }
        if let Ok(port) = env::var("DB_PORT") {
            if let Ok(parsed_port) = port.parse() {
                self.database.port = parsed_port;
            }
        }
        if let Ok(debug_mode) = env::var("DEBUG_MODE") {
            if let Ok(parsed_debug) = debug_mode.parse() {
                self.debug_mode = parsed_debug;
            }
        }
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database.host.is_empty() {
            errors.push("Database host cannot be empty".to_string());
        }
        if self.database.port == 0 {
            errors.push("Database port cannot be zero".to_string());
        }
        if self.server.port == 0 {
            errors.push("Server port cannot be zero".to_string());
        }
        if self.server.max_connections == 0 {
            errors.push("Max connections cannot be zero".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
        assert!(!config.debug_mode);
    }

    #[test]
    fn test_config_from_file() {
        let config_content = r#"
            [database]
            host = "db.example.com"
            port = 3306
            username = "admin"
            password = "secret"
            database_name = "production"

            [server]
            address = "127.0.0.1"
            port = 3000
            max_connections = 500

            debug_mode = true
            log_level = "debug"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, config_content.as_bytes()).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.database.host, "db.example.com");
        assert_eq!(config.database.port, 3306);
        assert_eq!(config.server.port, 3000);
        assert!(config.debug_mode);
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.database.port = 0;
        config.server.max_connections = 0;

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2);
    }
}