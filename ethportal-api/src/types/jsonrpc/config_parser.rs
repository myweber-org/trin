
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub numeric_values: HashMap<String, f64>,
    pub flags: HashMap<String, bool>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            numeric_values: HashMap::new(),
            flags: HashMap::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", trimmed));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim();

            if let Ok(num) = value.parse::<f64>() {
                config.numeric_values.insert(key, num);
            } else if let Ok(flag) = value.parse::<bool>() {
                config.flags.insert(key, flag);
            } else {
                config.settings.insert(key, value.to_string());
            }
        }

        Ok(config)
    }

    pub fn get_string(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.numeric_values.get(key).copied()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.flags.get(key).copied()
    }

    pub fn validate_required(&self, required_keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();
        
        for key in required_keys {
            if !self.settings.contains_key(*key) && 
               !self.numeric_values.contains_key(*key) && 
               !self.flags.contains_key(*key) {
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "server_host = localhost").unwrap();
        writeln!(temp_file, "server_port = 8080").unwrap();
        writeln!(temp_file, "enable_ssl = true").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "timeout = 30.5").unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.get_string("server_host"), Some(&"localhost".to_string()));
        assert_eq!(config.get_number("server_port"), Some(8080.0));
        assert_eq!(config.get_number("timeout"), Some(30.5));
        assert_eq!(config.get_bool("enable_ssl"), Some(true));
    }

    #[test]
    fn test_validation() {
        let mut config = Config::new();
        config.settings.insert("host".to_string(), "localhost".to_string());
        config.numeric_values.insert("port".to_string(), 8080.0);
        
        let result = config.validate_required(&["host", "port", "missing"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec!["missing"]);
    }
}