
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
                let mut processed_value = value.trim().to_string();
                
                processed_value = Self::substitute_env_vars(&processed_value);
                values.insert(key, processed_value);
            }
        }
        
        Ok(Config { values })
    }
    
    fn substitute_env_vars(input: &str) -> String {
        let mut result = input.to_string();
        
        for (key, value) in env::vars() {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, &value);
        }
        
        result
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
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "MAX_CONNECTIONS=10").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "TIMEOUT=30").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/db");
        assert_eq!(config.get("MAX_CONNECTIONS").unwrap(), "10");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
        assert!(config.get("NONEXISTENT").is_none());
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "8080");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "PORT=${{APP_PORT}}").unwrap();
        writeln!(temp_file, "HOST=localhost:${{APP_PORT}}").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("HOST").unwrap(), "localhost:8080");
    }
    
    #[test]
    fn test_get_or_default() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "EXISTING_KEY=actual_value").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get_or_default("EXISTING_KEY", "default"), "actual_value");
        assert_eq!(config.get_or_default("MISSING_KEY", "default_value"), "default_value");
    }
}