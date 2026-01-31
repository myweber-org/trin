use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut values = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid line format: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::resolve_value(&raw_value)?;

            values.insert(key, value);
        }

        Ok(Config { values })
    }

    fn resolve_value(raw_value: &str) -> Result<String, String> {
        if raw_value.starts_with("${") && raw_value.ends_with('}') {
            let var_name = &raw_value[2..raw_value.len() - 1];
            env::var(var_name).map_err(|_| format!("Environment variable not found: {}", var_name))
        } else {
            Ok(raw_value.to_string())
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_var_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DB_HOST=localhost").unwrap();
        writeln!(file, "DB_PASS=${{DB_PASSWORD}}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DB_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DB_PASS"), Some(&"secret123".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub rotation: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_connections: 100,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                pool_size: 10,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                rotation: "daily".to_string(),
            },
        }
    }
}

pub struct ConfigParser;

impl ConfigParser {
    pub fn from_file(path: &str) -> Result<AppConfig, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        
        Self::validate(&config)?;
        Ok(config)
    }
    
    pub fn from_env() -> Result<AppConfig, String> {
        let mut config = AppConfig::default();
        
        if let Ok(host) = std::env::var("SERVER_HOST") {
            config.server.host = host;
        }
        
        if let Ok(port) = std::env::var("SERVER_PORT") {
            config.server.port = port.parse()
                .map_err(|e| format!("Invalid port value: {}", e))?;
        }
        
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database.url = db_url;
        }
        
        Self::validate(&config)?;
        Ok(config)
    }
    
    pub fn merge_with_default(mut config: AppConfig) -> AppConfig {
        let default = AppConfig::default();
        
        if config.server.host.is_empty() {
            config.server.host = default.server.host;
        }
        
        if config.server.port == 0 {
            config.server.port = default.server.port;
        }
        
        if config.server.max_connections == 0 {
            config.server.max_connections = default.server.max_connections;
        }
        
        if config.database.url.is_empty() {
            config.database.url = default.database.url;
        }
        
        if config.database.pool_size == 0 {
            config.database.pool_size = default.database.pool_size;
        }
        
        if config.database.timeout_seconds == 0 {
            config.database.timeout_seconds = default.database.timeout_seconds;
        }
        
        if config.logging.level.is_empty() {
            config.logging.level = default.logging.level;
        }
        
        if config.logging.rotation.is_empty() {
            config.logging.rotation = default.logging.rotation;
        }
        
        config
    }
    
    fn validate(config: &AppConfig) -> Result<(), String> {
        if config.server.port > 65535 {
            return Err("Port number must be between 1 and 65535".to_string());
        }
        
        if config.server.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }
        
        if config.database.pool_size == 0 {
            return Err("Database pool size must be greater than 0".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&config.logging.level.as_str()) {
            return Err(format!("Invalid log level: {}. Valid levels are: {:?}", 
                config.logging.level, valid_log_levels));
        }
        
        let valid_rotations = ["hourly", "daily", "weekly", "monthly"];
        if !valid_rotations.contains(&config.logging.rotation.as_str()) {
            return Err(format!("Invalid log rotation: {}. Valid rotations are: {:?}", 
                config.logging.rotation, valid_rotations));
        }
        
        Ok(())
    }
    
    pub fn to_toml(&self, config: &AppConfig) -> Result<String, String> {
        toml::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))
    }
    
    pub fn generate_example_config() -> String {
        let example_config = AppConfig::default();
        toml::to_string_pretty(&example_config)
            .unwrap_or_else(|_| "# Error generating example config".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.pool_size, 10);
        assert_eq!(config.logging.level, "info");
    }
    
    #[test]
    fn test_config_from_file() {
        let toml_content = r#"
            [server]
            host = "0.0.0.0"
            port = 3000
            max_connections = 500
            
            [database]
            url = "postgresql://prod:5432/appdb"
            pool_size = 20
            timeout_seconds = 60
            
            [logging]
            level = "debug"
            file_path = "/var/log/app.log"
            rotation = "daily"
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), toml_content).unwrap();
        
        let result = ConfigParser::from_file(file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.logging.level, "debug");
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server.port = 70000; // Invalid port
        
        let result = ConfigParser::validate(&config);
        assert!(result.is_err());
        
        config.server.port = 8080;
        config.logging.level = "invalid".to_string();
        
        let result = ConfigParser::validate(&config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_merge_with_default() {
        let mut config = AppConfig::default();
        config.server.host = String::new();
        config.server.port = 0;
        
        let merged = ConfigParser::merge_with_default(config);
        assert_eq!(merged.server.host, "127.0.0.1");
        assert_eq!(merged.server.port, 8080);
    }
}