use std::env;
use std::fs;
use std::collections::HashMap;

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
                let value = value.trim().to_string();
                values.insert(key, value);
            }
        }
        
        Ok(Config { values })
    }
    
    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }
    
    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        env::var(key).is_ok() || self.values.contains_key(key)
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
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some("postgres://localhost/test".to_string()));
        assert_eq!(config.get("API_KEY"), Some("secret123".to_string()));
        assert_eq!(config.get("NON_EXISTENT"), None);
    }
    
    #[test]
    fn test_env_override() {
        env::set_var("TEST_KEY", "env_value");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "TEST_KEY=file_value").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("TEST_KEY"), Some("env_value".to_string()));
        
        env::remove_var("TEST_KEY");
    }
}