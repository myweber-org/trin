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
            
            if let Some(separator) = trimmed.find('=') {
                let key = trimmed[..separator].trim().to_string();
                let mut value = trimmed[separator + 1..].trim().to_string();
                
                value = var_pattern.replace_all(&value, |caps: ®ex::Captures| {
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
        self.values.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = r#"
            server_host=localhost
            server_port=8080
            debug_mode=true
        "#;
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("server_host"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("server_port"), Some(&"8080".to_string()));
        assert_eq!(parser.get("debug_mode"), Some(&"true".to_string()));
        assert_eq!(parser.get("missing_key"), None);
    }
    
    #[test]
    fn test_env_variable_substitution() {
        env::set_var("APP_DB_HOST", "postgres.local");
        
        let mut parser = ConfigParser::new();
        let config = r#"
            database_host=${APP_DB_HOST}
            fallback_value=${NONEXISTENT_VAR}
        "#;
        
        parser.load_from_str(config).unwrap();
        
        assert_eq!(parser.get("database_host"), Some(&"postgres.local".to_string()));
        assert_eq!(parser.get("fallback_value"), Some(&"".to_string()));
    }
}