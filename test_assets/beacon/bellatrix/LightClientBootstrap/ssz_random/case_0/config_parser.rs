use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub features: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut database_url = None;
        let mut max_connections = 10;
        let mut timeout_seconds = 30;
        let mut features = Vec::new();
        let mut metadata = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "database_url" => database_url = Some(value.to_string()),
                "max_connections" => {
                    max_connections = value.parse()
                        .map_err(|_| format!("Invalid number for max_connections: {}", value))?
                }
                "timeout_seconds" => {
                    timeout_seconds = value.parse()
                        .map_err(|_| format!("Invalid number for timeout_seconds: {}", value))?
                }
                "feature" => features.push(value.to_string()),
                _ => {
                    metadata.insert(key.to_string(), value.to_string());
                }
            }
        }

        let database_url = database_url
            .ok_or_else(|| "Missing required field: database_url".to_string())?;

        Ok(Config {
            database_url,
            max_connections,
            timeout_seconds,
            features,
            metadata,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        if self.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }

        if self.timeout_seconds > 3600 {
            return Err("Timeout cannot exceed 3600 seconds".to_string());
        }

        Ok(())
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "database_url=postgres://localhost/db").unwrap();
        writeln!(file, "max_connections=20").unwrap();
        writeln!(file, "timeout_seconds=60").unwrap();
        writeln!(file, "feature=caching").unwrap();
        writeln!(file, "feature=logging").unwrap();
        writeln!(file, "custom_key=custom_value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.features, vec!["caching", "logging"]);
        assert_eq!(config.get_metadata("custom_key"), Some(&"custom_value".to_string()));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "max_connections=20").unwrap();

        let result = Config::from_file(file.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required field"));
    }

    #[test]
    fn test_invalid_number() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "database_url=postgres://localhost/db").unwrap();
        writeln!(file, "max_connections=invalid").unwrap();

        let result = Config::from_file(file.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid number"));
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut values = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map_or(default.to_string(), |v| v.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "PORT=8080").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/db");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("API_KEY", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "KEY=$API_KEY").unwrap();
        writeln!(file, "OTHER=plain_value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("KEY").unwrap(), "secret123");
        assert_eq!(config.get("OTHER").unwrap(), "plain_value");
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub workers: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub features: HashMap<String, bool>,
    pub log_level: String,
}

pub struct ConfigParser;

impl ConfigParser {
    pub fn load_from_file(path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_yaml::from_str(&content)?;
        
        Self::substitute_env_vars(&mut config);
        Ok(config)
    }
    
    fn substitute_env_vars(config: &mut AppConfig) {
        config.database.host = Self::get_env_or_default("DB_HOST", &config.database.host);
        config.database.port = Self::get_env_u16_or_default("DB_PORT", config.database.port);
        config.database.username = Self::get_env_or_default("DB_USER", &config.database.username);
        config.database.password = Self::get_env_or_default("DB_PASS", &config.database.password);
        config.database.database = Self::get_env_or_default("DB_NAME", &config.database.database);
        
        config.server.address = Self::get_env_or_default("SERVER_ADDR", &config.server.address);
        config.server.port = Self::get_env_u16_or_default("SERVER_PORT", config.server.port);
        config.server.workers = Self::get_env_usize_or_default("SERVER_WORKERS", config.server.workers);
        
        config.log_level = Self::get_env_or_default("LOG_LEVEL", &config.log_level);
    }
    
    fn get_env_or_default(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }
    
    fn get_env_u16_or_default(key: &str, default: u16) -> u16 {
        env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }
    
    fn get_env_usize_or_default(key: &str, default: usize) -> usize {
        env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }
    
    pub fn validate_config(config: &AppConfig) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if config.server.port == 0 {
            errors.push("Server port cannot be zero".to_string());
        }
        
        if config.server.workers == 0 {
            errors.push("Number of workers cannot be zero".to_string());
        }
        
        if config.database.host.is_empty() {
            errors.push("Database host cannot be empty".to_string());
        }
        
        if config.database.port == 0 {
            errors.push("Database port cannot be zero".to_string());
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
    fn test_config_loading() {
        let yaml_content = r#"
server:
  address: "127.0.0.1"
  port: 8080
  workers: 4
  timeout_seconds: 30

database:
  host: "localhost"
  port: 5432
  username: "postgres"
  password: "secret"
  database: "app_db"

features:
  caching: true
  metrics: false

log_level: "info"
"#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), yaml_content).unwrap();
        
        let config = ConfigParser::load_from_file(file.path().to_str().unwrap());
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.features.get("caching"), Some(&true));
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "env-host");
        env::set_var("SERVER_PORT", "9090");
        
        let yaml_content = r#"
server:
  address: "127.0.0.1"
  port: 8080
  workers: 4
  timeout_seconds: 30

database:
  host: "default-host"
  port: 5432
  username: "postgres"
  password: "secret"
  database: "app_db"

features: {}

log_level: "info"
"#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), yaml_content).unwrap();
        
        let config = ConfigParser::load_from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database.host, "env-host");
        assert_eq!(config.server.port, 9090);
        
        env::remove_var("DB_HOST");
        env::remove_var("SERVER_PORT");
    }
}