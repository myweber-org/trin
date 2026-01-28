use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        let var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some(equals_pos) = trimmed.find('=') {
                let key = trimmed[..equals_pos].trim().to_string();
                let mut value = trimmed[equals_pos + 1..].trim().to_string();
                
                value = var_pattern.replace_all(&value, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    env::var(var_name).unwrap_or_else(|_| String::new())
                }).to_string();
                
                self.values.insert(key, value);
            }
        }
        
        Ok(())
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
    
    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = r#"
            database_host=localhost
            database_port=5432
            # This is a comment
            api_key=secret_value
        "#;
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("database_host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("database_port"), Some(&"5432".to_string()));
        assert_eq!(parser.get("api_key"), Some(&"secret_value".to_string()));
        assert_eq!(parser.get("nonexistent"), None);
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "8080");
        env::set_var("DB_HOST", "postgres-server");
        
        let mut parser = ConfigParser::new();
        let config = r#"
            port=${APP_PORT}
            host=${DB_HOST}
            missing=${UNDEFINED_VAR}
        "#;
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("port"), Some(&"8080".to_string()));
        assert_eq!(parser.get("host"), Some(&"postgres-server".to_string()));
        assert_eq!(parser.get("missing"), Some(&"".to_string()));
    }
}