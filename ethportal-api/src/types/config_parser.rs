
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl Config {
    pub fn new() -> Result<Self, String> {
        let mut config = HashMap::new();
        
        // Load from file if exists
        if let Ok(content) = fs::read_to_string("config.toml") {
            let file_config: HashMap<String, String> = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse config file: {}", e))?;
            config.extend(file_config);
        }
        
        // Override with environment variables
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                config.insert(config_key, value);
            }
        }
        
        // Build config with defaults
        let database_url = config
            .get("database_url")
            .cloned()
            .unwrap_or_else(|| "postgres://localhost:5432/app".to_string());
            
        let server_port = config
            .get("server_port")
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
            
        let log_level = config
            .get("log_level")
            .cloned()
            .unwrap_or_else(|| "info".to_string());
            
        let cache_ttl = config
            .get("cache_ttl")
            .and_then(|t| t.parse().ok())
            .unwrap_or(300);
        
        Ok(Self {
            database_url,
            server_port,
            log_level,
            cache_ttl,
        })
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.database_url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }
        
        if self.server_port == 0 {
            errors.push("Server port must be greater than 0".to_string());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            errors.push(format!(
                "Invalid log level '{}'. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
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
    fn test_default_config() {
        let config = Config::new().unwrap();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = Config {
            database_url: "".to_string(),
            server_port: 0,
            log_level: "invalid".to_string(),
            cache_ttl: 300,
        };
        
        let errors = config.validate().unwrap_err();
        assert_eq!(errors.len(), 3);
    }
}