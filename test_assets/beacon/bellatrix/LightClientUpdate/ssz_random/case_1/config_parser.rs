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
        let re = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+?)\s*$").unwrap();
        
        for line in content.lines() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            
            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();
                
                self.process_environment_variables(&mut value);
                self.values.insert(key, value);
            } else {
                return Err(format!("Invalid configuration line: {}", line));
            }
        }
        
        Ok(())
    }
    
    fn process_environment_variables(&self, value: &mut String) {
        let env_regex = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
        
        *value = env_regex.replace_all(value, |caps: &regex::Captures| {
            let var_name = &caps[1];
            env::var(var_name).unwrap_or_else(|_| String::new())
        }).to_string();
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
            DATABASE_HOST=localhost
            DATABASE_PORT=5432
            # This is a comment
            API_KEY=secret123
        "#;
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(parser.get("API_KEY"), Some(&"secret123".to_string()));
    }
    
    #[test]
    fn test_environment_substitution() {
        env::set_var("APP_ENV", "production");
        
        let mut parser = ConfigParser::new();
        let config = r#"ENVIRONMENT=${APP_ENV}"#;
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("ENVIRONMENT"), Some(&"production".to_string()));
    }
}