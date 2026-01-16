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
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
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
        assert_eq!(config.get("DATABASE_URL"), Some(&"postgres://localhost/db".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("API_KEY", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "KEY=$API_KEY").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("KEY"), Some(&"secret123".to_string()));
    }
}use serde::{Deserialize, Serialize};
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub features: HashMap<String, bool>,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = serde_yaml::from_str(&content)?;
        config.apply_environment_overrides();
        Ok(config)
    }

    fn apply_environment_overrides(&mut self) {
        if let Ok(addr) = env::var("SERVER_ADDRESS") {
            self.server.address = addr;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                self.server.port = port_num;
            }
        }
        if let Ok(db_host) = env::var("DB_HOST") {
            self.database.host = db_host;
        }
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push("Server port cannot be zero".to_string());
        }
        if self.database.host.is_empty() {
            errors.push("Database host cannot be empty".to_string());
        }
        if !["error", "warn", "info", "debug", "trace"].contains(&self.log_level.as_str()) {
            errors.push(format!("Invalid log level: {}", self.log_level));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn to_yaml(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_yaml::to_string(self)?)
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
database:
  host: "localhost"
  port: 5432
  username: "postgres"
  password: "secret"
  database: "appdb"
features:
  caching: true
  analytics: false
log_level: "info"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        use std::io::Write;
        temp_file.write_all(yaml_content.as_bytes()).unwrap();

        let config = AppConfig::from_file(temp_file.path().to_str().unwrap());
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_environment_override() {
        env::set_var("DB_HOST", "prod-db.example.com");
        env::set_var("LOG_LEVEL", "debug");

        let yaml_content = r#"
server:
  address: "127.0.0.1"
  port: 8080
  workers: 4
database:
  host: "localhost"
  port: 5432
  username: "postgres"
  password: "secret"
  database: "appdb"
features: {}
log_level: "info"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        use std::io::Write;
        temp_file.write_all(yaml_content.as_bytes()).unwrap();

        let config = AppConfig::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database.host, "prod-db.example.com");
        assert_eq!(config.log_level, "debug");

        env::remove_var("DB_HOST");
        env::remove_var("LOG_LEVEL");
    }

    #[test]
    fn test_config_validation() {
        let invalid_config = AppConfig {
            server: ServerConfig {
                address: "127.0.0.1".to_string(),
                port: 0,
                workers: 4,
            },
            database: DatabaseConfig {
                host: "".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "secret".to_string(),
                database: "appdb".to_string(),
            },
            features: HashMap::new(),
            log_level: "invalid".to_string(),
        };

        let result = invalid_config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3);
    }
}