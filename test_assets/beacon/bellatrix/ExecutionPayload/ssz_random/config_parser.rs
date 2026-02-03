use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

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
    pub file_path: String,
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
                url: "postgresql://localhost:5432/appdb".to_string(),
                max_connections: 10,
                pool_timeout_seconds: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "logs/app.log".to_string(),
                max_file_size_mb: 100,
            },
        }
    }
}

pub fn load_config(config_path: &str) -> Result<AppConfig, String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: AppConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    validate_config(&config)?;
    Ok(config)
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }
    
    if config.database.max_connections == 0 {
        return Err("Database max connections cannot be 0".to_string());
    }
    
    if config.logging.max_file_size_mb == 0 {
        return Err("Log file max size cannot be 0".to_string());
    }
    
    Ok(())
}

pub fn generate_default_config(config_path: &str) -> Result<(), String> {
    let default_config = AppConfig::default();
    let toml_content = toml::to_string_pretty(&default_config)
        .map_err(|e| format!("Failed to serialize default config: {}", e))?;
    
    fs::write(config_path, toml_content)
        .map_err(|e| format!("Failed to write default config: {}", e))?;
    
    Ok(())
}

pub fn merge_configs(base: AppConfig, overrides: HashMap<String, String>) -> AppConfig {
    let mut merged = base;
    
    for (key, value) in overrides {
        match key.as_str() {
            "server.host" => merged.server.host = value,
            "server.port" => if let Ok(port) = value.parse() { merged.server.port = port },
            "server.timeout_seconds" => if let Ok(timeout) = value.parse() { merged.server.timeout_seconds = timeout },
            "database.url" => merged.database.url = value,
            "database.max_connections" => if let Ok(conns) = value.parse() { merged.database.max_connections = conns },
            "database.pool_timeout_seconds" => if let Ok(timeout) = value.parse() { merged.database.pool_timeout_seconds = timeout },
            "logging.level" => merged.logging.level = value,
            "logging.file_path" => merged.logging.file_path = value,
            "logging.max_file_size_mb" => if let Ok(size) = value.parse() { merged.logging.max_file_size_mb = size },
            _ => (),
        }
    }
    
    merged
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
                let value = Self::resolve_value(value.trim());
                values.insert(key, value);
            }
        }

        Ok(Config { values })
    }

    fn resolve_value(input: &str) -> String {
        if let Some(var_name) = input.strip_prefix("${").and_then(|s| s.strip_suffix('}')) {
            env::var(var_name).unwrap_or_else(|_| input.to_string())
        } else {
            input.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or(default.to_string())
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
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_variable_resolution() {
        env::set_var("DB_PASSWORD", "secret123");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=${DB_PASSWORD}").unwrap();
        writeln!(file, "HOST=localhost").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub defaults: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            defaults: HashMap::from([
                ("timeout".to_string(), "30".to_string()),
                ("retries".to_string(), "3".to_string()),
                ("log_level".to_string(), "info".to_string()),
            ]),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            if value.is_empty() {
                return Err(format!("Empty value for key: {}", key));
            }

            self.settings.insert(key, value);
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key).or_else(|| self.defaults.get(key))
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }

    pub fn validate_required(&self, keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();

        for key in keys {
            if !self.settings.contains_key(*key) && !self.defaults.contains_key(*key) {
                missing.push(format!("Required config key '{}' is missing", key));
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    pub fn merge(&mut self, other: Config) {
        for (key, value) in other.settings {
            self.settings.insert(key, value);
        }
        for (key, value) in other.defaults {
            self.defaults.insert(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "host=localhost").unwrap();
        writeln!(temp_file, "port=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "timeout=60").unwrap();

        let result = config.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"60".to_string()));
        assert_eq!(config.get("retries"), Some(&"3".to_string()));
    }

    #[test]
    fn test_missing_key_falls_back_to_default() {
        let config = Config::new();
        assert_eq!(config.get("log_level"), Some(&"info".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        config.settings.insert("host".to_string(), "localhost".to_string());
        
        let result = config.validate_required(&["host", "api_key"]);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&"Required config key 'api_key' is missing".to_string()));
    }

    #[test]
    fn test_get_with_custom_default() {
        let config = Config::new();
        assert_eq!(config.get_with_default("timeout", "10"), "30");
        assert_eq!(config.get_with_default("unknown", "default_value"), "default_value");
    }
}