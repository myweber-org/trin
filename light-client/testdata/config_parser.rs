use std::collections::HashMap;
use std::fs;
use std::io;

#[derive(Debug)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, io::Error> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, io::Error> {
        let mut config = Config::new();
        
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid format at line {}", line_num + 1)
                ));
            }
            
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            
            if key.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Empty key at line {}", line_num + 1)
                ));
            }
            
            config.settings.insert(key, value);
        }
        
        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.settings.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }

    pub fn validate_required(&self, required_keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();
        
        for key in required_keys {
            if !self.settings.contains_key(*key) {
                missing.push(key.to_string());
            }
        }
        
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let content = "host=localhost\nport=8080\n# This is a comment\n\ndebug=true";
        let config = Config::parse(content).unwrap();
        
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("debug"), Some(&"true".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_parse_invalid_format() {
        let content = "host=localhost\ninvalid_line\nport=8080";
        let result = Config::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_required() {
        let content = "host=localhost\nport=8080";
        let config = Config::parse(content).unwrap();
        
        let result = config.validate_required(&["host", "port"]);
        assert!(result.is_ok());
        
        let result = config.validate_required(&["host", "port", "missing"]);
        assert!(result.is_err());
    }
}