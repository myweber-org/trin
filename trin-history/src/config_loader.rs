use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                config.values.insert(key, value);
            }
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key) || env::var(key).is_ok()
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
        writeln!(temp_file, "DATABASE_URL=postgres://localhost").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some("postgres://localhost".to_string()));
        assert_eq!(config.get("API_KEY"), Some("secret123".to_string()));
        assert_eq!(config.get("NON_EXISTENT"), None);
    }

    #[test]
    fn test_env_override() {
        env::set_var("TEST_ENV_VAR", "env_value");
        let config = Config::new();
        assert_eq!(config.get("TEST_ENV_VAR"), Some("env_value".to_string()));
        env::remove_var("TEST_ENV_VAR");
    }

    #[test]
    fn test_default_value() {
        let config = Config::new();
        assert_eq!(config.get_with_default("MISSING_KEY", "default_value"), "default_value");
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        let mut values = HashMap::new();
        
        // Load from .env file if exists
        if let Ok(contents) = fs::read_to_string(".env") {
            for line in contents.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                
                if let Some((key, value)) = trimmed.split_once('=') {
                    values.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
        
        // Override with system environment variables
        for (key, value) in env::vars() {
            values.insert(key, value);
        }
        
        Config { values }
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
    
    pub fn require(&self, key: &str) -> Result<String, String> {
        self.values.get(key)
            .map(|s| s.clone())
            .ok_or_else(|| format!("Required configuration '{}' not found", key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_config_loading() {
        env::set_var("TEST_VAR", "env_value");
        
        let config = Config::new();
        
        // Test environment variable
        assert_eq!(config.get("TEST_VAR"), Some(&"env_value".to_string()));
        
        // Test default value
        assert_eq!(config.get_or_default("NON_EXISTENT", "default"), "default");
        
        // Test required value
        let result = config.require("TEST_VAR");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "env_value");
        
        // Test missing required value
        let result = config.require("MISSING_VAR");
        assert!(result.is_err());
    }
}