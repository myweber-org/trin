use std::collections::HashMap;
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
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                self.settings.insert(key, value);
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key).or_else(|| self.defaults.get(key))
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for (key, value) in &self.settings {
            match key.as_str() {
                "timeout" | "retries" => {
                    if value.parse::<u32>().is_err() {
                        errors.push(format!("{} must be a positive integer", key));
                    }
                }
                "log_level" => {
                    let valid_levels = ["error", "warn", "info", "debug", "trace"];
                    if !valid_levels.contains(&value.as_str()) {
                        errors.push(format!("{} must be one of: {:?}", key, valid_levels));
                    }
                }
                _ => {}
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn merge_defaults(&mut self) {
        for (key, value) in &self.defaults {
            self.settings.entry(key.clone()).or_insert(value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "timeout=60\nretries=5\n# comment\nlog_level=debug").unwrap();

        let result = config.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(config.get("timeout"), Some(&"60".to_string()));
        assert_eq!(config.get("log_level"), Some(&"debug".to_string()));
    }

    #[test]
    fn test_default_values() {
        let config = Config::new();
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        config.settings.insert("timeout".to_string(), "invalid".to_string());
        config.settings.insert("log_level".to_string(), "invalid".to_string());

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2);
    }
}use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub pool_size: u32,
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
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
    pub enable_debug: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "".to_string(),
            database: "app_db".to_string(),
            pool_size: 10,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            address: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
            timeout_seconds: 30,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            log_level: "info".to_string(),
            enable_debug: false,
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&config_str)?;
    
    validate_config(&config)?;
    
    Ok(config)
}

pub fn load_config_with_defaults<P: AsRef<Path>>(path: P) -> AppConfig {
    match load_config(path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}. Using defaults.", e);
            AppConfig::default()
        }
    }
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.database.host.is_empty() {
        return Err("Database host cannot be empty".to_string());
    }
    
    if config.database.port == 0 {
        return Err("Database port cannot be zero".to_string());
    }
    
    if config.database.pool_size == 0 {
        return Err("Database pool size must be greater than zero".to_string());
    }
    
    if config.server.port == 0 {
        return Err("Server port cannot be zero".to_string());
    }
    
    if config.server.workers == 0 {
        return Err("Server workers must be greater than zero".to_string());
    }
    
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.log_level.as_str()) {
        return Err(format!("Invalid log level: {}. Must be one of: {:?}", 
            config.log_level, valid_log_levels));
    }
    
    Ok(())
}

pub fn save_config<P: AsRef<Path>>(config: &AppConfig, path: P) -> Result<(), Box<dyn std::error::Error>> {
    validate_config(config)?;
    
    let config_str = toml::to_string_pretty(config)?;
    fs::write(path, config_str)?;
    
    Ok(())
}

pub fn generate_default_config<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let default_config = AppConfig::default();
    save_config(&default_config, path)
}use std::collections::HashMap;
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
                return Err(format!("Invalid config line: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let raw_value = parts[1].trim().to_string();
            let value = Self::resolve_value(&raw_value);

            values.insert(key, value);
        }

        Ok(Config { values })
    }

    fn resolve_value(raw_value: &str) -> String {
        if raw_value.starts_with("${") && raw_value.ends_with('}') {
            let var_name = &raw_value[2..raw_value.len() - 1];
            env::var(var_name).unwrap_or_else(|_| raw_value.to_string())
        } else {
            raw_value.to_string()
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
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DB_HOST=localhost").unwrap();
        writeln!(file, "DB_PASS=${{DB_PASSWORD}}").unwrap();
        writeln!(file, "NO_ENV=${{UNDEFINED_VAR}}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DB_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DB_PASS"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NO_ENV"), Some(&"${UNDEFINED_VAR}".to_string()));
    }

    #[test]
    fn test_invalid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "INVALID_LINE").unwrap();

        let result = Config::from_file(file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}