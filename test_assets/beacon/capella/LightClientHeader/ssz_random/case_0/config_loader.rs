use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config_map = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                config_map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = Self::get_value(map, "DATABASE_URL")?;
        let port_str = Self::get_value(map, "PORT")?;
        let port = port_str.parse::<u16>()?;
        let debug_str = Self::get_value(map, "DEBUG").unwrap_or_else(|_| "false".to_string());
        let debug_mode = debug_str.to_lowercase() == "true";

        let mut api_keys = HashMap::new();
        for (key, value) in map {
            if key.starts_with("API_KEY_") {
                api_keys.insert(key.clone(), value.clone());
            }
        }

        Ok(Config {
            database_url,
            port,
            debug_mode,
            api_keys,
        })
    }

    fn get_value(map: &HashMap<String, String>, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(value) = env::var(key).ok().or_else(|| map.get(key).cloned()) {
            Ok(value)
        } else {
            Err(format!("Missing required configuration: {}", key).into())
        }
    }

    pub fn get_api_key(&self, service: &str) -> Option<&String> {
        let key_name = format!("API_KEY_{}", service.to_uppercase());
        self.api_keys.get(&key_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(temp_file, "PORT=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "DEBUG=true").unwrap();
        writeln!(temp_file, "API_KEY_WEATHER=abc123").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.port, 8080);
        assert!(config.debug_mode);
        assert_eq!(config.get_api_key("weather"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_env_var_override() {
        env::set_var("DATABASE_URL", "postgres://prod/db");
        
        let mut map = HashMap::new();
        map.insert("PORT".to_string(), "3000".to_string());
        
        let config = Config::from_map(&map).unwrap();
        assert_eq!(config.database_url, "postgres://prod/db");
        assert_eq!(config.port, 3000);
        assert!(!config.debug_mode);
        
        env::remove_var("DATABASE_URL");
    }
}use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u32,
}

impl AppConfig {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;

        let mut config: AppConfig = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        if let Ok(env_port) = env::var("SERVER_PORT") {
            if let Ok(port) = env_port.parse::<u16>() {
                config.server_port = port;
            }
        }

        if let Ok(env_db_url) = env::var("DATABASE_URL") {
            config.database_url = env_db_url;
        }

        if let Ok(env_log_level) = env::var("LOG_LEVEL") {
            config.log_level = env_log_level.to_uppercase();
        }

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }

        let valid_log_levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }

        if self.cache_ttl > 86400 {
            return Err("Cache TTL cannot exceed 24 hours".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_validation() {
        let config = AppConfig {
            server_port: 8080,
            database_url: "postgres://localhost/db".to_string(),
            log_level: "INFO".to_string(),
            cache_ttl: 3600,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_log_level() {
        let config = AppConfig {
            server_port: 8080,
            database_url: "postgres://localhost/db".to_string(),
            log_level: "INVALID".to_string(),
            cache_ttl: 3600,
        };

        assert!(config.validate().is_err());
    }
}