use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&config_str)?;
        
        config.apply_environment_overrides();
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                self.server_port = port_num;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
        
        if let Ok(cache_ttl) = env::var("CACHE_TTL") {
            if let Ok(ttl) = cache_ttl.parse() {
                self.cache_ttl = ttl;
            }
        }
    }
    
    pub fn default() -> Self {
        Self {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/app_db"),
            log_level: String::from("info"),
            cache_ttl: 3600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
            server_port = 3000
            database_url = "postgresql://localhost:5432/test_db"
            log_level = "debug"
            cache_ttl = 1800
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.database_url, "postgresql://localhost:5432/test_db");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_ttl, 1800);
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("SERVER_PORT", "9090");
        env::set_var("LOG_LEVEL", "trace");
        
        let toml_content = r#"
            server_port = 3000
            database_url = "postgresql://localhost:5432/test_db"
            log_level = "debug"
            cache_ttl = 1800
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.log_level, "trace");
        
        env::remove_var("SERVER_PORT");
        env::remove_var("LOG_LEVEL");
    }
}
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub thresholds: HashMap<String, f64>,
    pub enabled: bool,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        Self::parse(&content)
    }

    fn parse(content: &str) -> Result<Self, String> {
        let mut settings = HashMap::new();
        let mut thresholds = HashMap::new();
        let mut enabled = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid line format: {}", line));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            match key.as_str() {
                "enabled" => {
                    enabled = value.parse()
                        .map_err(|_| format!("Invalid boolean value for 'enabled': {}", value))?;
                }
                key if key.starts_with("threshold_") => {
                    let threshold_value: f64 = value.parse()
                        .map_err(|_| format!("Invalid float value for '{}': {}", key, value))?;
                    thresholds.insert(key, threshold_value);
                }
                _ => {
                    settings.insert(key, value);
                }
            }
        }

        Ok(Config {
            settings,
            thresholds,
            enabled,
        })
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_threshold(&self, key: &str) -> Option<&f64> {
        self.thresholds.get(key)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if !self.enabled && !self.thresholds.is_empty() {
            errors.push("Thresholds defined but service is disabled".to_string());
        }

        for (key, value) in &self.thresholds {
            if *value < 0.0 || *value > 100.0 {
                errors.push(format!("Threshold '{}' must be between 0.0 and 100.0", key));
            }
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

    #[test]
    fn test_parse_valid_config() {
        let content = r#"
            server_host=localhost
            server_port=8080
            enabled=true
            threshold_cpu=80.5
            threshold_memory=90.0
        "#;

        let config = Config::parse(content).unwrap();
        assert_eq!(config.get_setting("server_host"), Some(&"localhost".to_string()));
        assert_eq!(config.get_setting("server_port"), Some(&"8080".to_string()));
        assert_eq!(config.get_threshold("threshold_cpu"), Some(&80.5));
        assert_eq!(config.get_threshold("threshold_memory"), Some(&90.0));
        assert!(config.enabled);
    }

    #[test]
    fn test_parse_invalid_boolean() {
        let content = "enabled=yes";
        let result = Config::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation() {
        let mut config = Config {
            settings: HashMap::new(),
            thresholds: HashMap::from([
                ("threshold_cpu".to_string(), 80.0),
                ("threshold_memory".to_string(), 150.0),
            ]),
            enabled: true,
        };

        let validation = config.validate();
        assert!(validation.is_err());
        let errors = validation.unwrap_err();
        assert!(errors.contains(&"Threshold 'threshold_memory' must be between 0.0 and 100.0".to_string()));

        config.enabled = false;
        let validation = config.validate();
        assert!(validation.is_err());
        let errors = validation.unwrap_err();
        assert!(errors.contains(&"Thresholds defined but service is disabled".to_string()));
    }
}