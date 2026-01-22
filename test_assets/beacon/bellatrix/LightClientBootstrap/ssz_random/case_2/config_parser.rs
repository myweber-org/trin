
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config_map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                config_map.insert(key, value);
            }
        }
        
        Self::from_map(&config_map)
    }
    
    pub fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = Self::get_value(map, "DATABASE_URL")
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://localhost:5432/mydb".to_string());
            
        let port = Self::get_value(map, "PORT")
            .or_else(|| env::var("PORT").ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);
            
        let debug_mode = Self::get_value(map, "DEBUG")
            .or_else(|| env::var("DEBUG").ok())
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);
            
        let api_keys = Self::get_value(map, "API_KEYS")
            .or_else(|| env::var("API_KEYS").ok())
            .map(|s| s.split(',').map(|k| k.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);
            
        Ok(Config {
            database_url,
            port,
            debug_mode,
            api_keys,
        })
    }
    
    fn get_value(map: &HashMap<String, String>, key: &str) -> Option<String> {
        map.get(key).cloned()
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.database_url.is_empty() {
            errors.push("DATABASE_URL cannot be empty".to_string());
        }
        
        if self.port == 0 {
            errors.push("PORT must be greater than 0".to_string());
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
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost:5432/test").unwrap();
        writeln!(temp_file, "PORT=3000").unwrap();
        writeln!(temp_file, "DEBUG=true").unwrap();
        writeln!(temp_file, "API_KEYS=key1,key2,key3").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost:5432/test");
        assert_eq!(config.port, 3000);
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.api_keys, vec!["key1", "key2", "key3"]);
    }
    
    #[test]
    fn test_config_defaults() {
        let map = HashMap::new();
        let config = Config::from_map(&map).unwrap();
        assert_eq!(config.database_url, "postgres://localhost:5432/mydb");
        assert_eq!(config.port, 8080);
        assert_eq!(config.debug_mode, false);
        assert!(config.api_keys.is_empty());
    }
    
    #[test]
    fn test_config_validation() {
        let mut map = HashMap::new();
        map.insert("DATABASE_URL".to_string(), "".to_string());
        map.insert("PORT".to_string(), "0".to_string());
        
        let config = Config::from_map(&map).unwrap();
        let validation_result = config.validate();
        assert!(validation_result.is_err());
        
        if let Err(errors) = validation_result {
            assert!(errors.contains(&"DATABASE_URL cannot be empty".to_string()));
            assert!(errors.contains(&"PORT must be greater than 0".to_string()));
        }
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
        if let Ok(host) = env::var("DB_HOST") {
            self.database.host = host;
        }
        
        if let Ok(port) = env::var("DB_PORT") {
            if let Ok(port_num) = port.parse() {
                self.database.port = port_num;
            }
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
        
        for (key, value) in env::vars() {
            if key.starts_with("FEATURE_") {
                let feature_name = key.trim_start_matches("FEATURE_").to_lowercase();
                if let Ok(enabled) = value.parse() {
                    self.features.insert(feature_name, enabled);
                }
            }
        }
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.server.port == 0 {
            errors.push("Server port cannot be 0".to_string());
        }
        
        if self.database.host.is_empty() {
            errors.push("Database host cannot be empty".to_string());
        }
        
        if self.database.port == 0 {
            errors.push("Database port cannot be 0".to_string());
        }
        
        if !["debug", "info", "warn", "error"].contains(&self.log_level.as_str()) {
            errors.push(format!("Invalid log level: {}", self.log_level));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn load_config_with_fallback(paths: &[&str]) -> Result<AppConfig, Box<dyn std::error::Error>> {
    for path in paths {
        if let Ok(config) = AppConfig::from_file(path) {
            if let Err(errors) = config.validate() {
                eprintln!("Configuration validation failed for {}: {:?}", path, errors);
                continue;
            }
            return Ok(config);
        }
    }
    
    Err("No valid configuration file found".into())
}