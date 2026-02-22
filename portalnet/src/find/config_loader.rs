use std::collections::HashMap;
use std::env;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        let mut values = HashMap::new();
        
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                values.insert(key.to_lowercase().replace("app_", ""), value);
            }
        }
        
        Config { values }
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }
    
    pub fn load_from_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        
        for line in content.lines() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                self.values.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        
        Ok(())
    }
    
    pub fn all(&self) -> &HashMap<String, String> {
        &self.values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_creation() {
        env::set_var("APP_DATABASE_URL", "postgres://localhost/test");
        env::set_var("APP_API_KEY", "secret123");
        
        let config = Config::new();
        
        assert_eq!(config.get("database_url"), Some(&"postgres://localhost/test".to_string()));
        assert_eq!(config.get("api_key"), Some(&"secret123".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }
    
    #[test]
    fn test_get_or_default() {
        let config = Config::new();
        
        assert_eq!(config.get_or_default("port", "8080"), "8080");
    }
}