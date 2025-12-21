use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub options: HashMap<String, bool>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            options: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::parse(&content)
    }

    fn parse(content: &str) -> Result<Self, String> {
        let mut config = Config::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid line: {}", line));
            }
            
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            
            if value == "true" || value == "false" {
                config.options.insert(key, value == "true");
            } else {
                config.settings.insert(key, value);
            }
        }
        
        Ok(config)
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_option(&self, key: &str) -> Option<bool> {
        self.options.get(key).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let content = r#"
            server_host = localhost
            server_port = 8080
            enable_logging = true
            debug_mode = false
        "#;
        
        let config = Config::parse(content).unwrap();
        assert_eq!(config.get_setting("server_host"), Some(&"localhost".to_string()));
        assert_eq!(config.get_setting("server_port"), Some(&"8080".to_string()));
        assert_eq!(config.get_option("enable_logging"), Some(true));
        assert_eq!(config.get_option("debug_mode"), Some(false));
    }
}