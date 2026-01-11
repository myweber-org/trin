
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
        let mut config = Config::new();

        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                
                if key.is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Empty key found in configuration"
                    ));
                }
                
                config.settings.insert(key, value);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid line format: {}", trimmed)
                ));
            }
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn validate_required(&self, required_keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();
        
        for &key in required_keys {
            if !self.settings.contains_key(key) {
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
    fn test_load_valid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "host=localhost").unwrap();
        writeln!(temp_file, "port=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "timeout=30").unwrap();

        let config = Config::load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_load_invalid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid_line_without_equals").unwrap();

        let result = Config::load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_required() {
        let mut config = Config::new();
        config.settings.insert("host".to_string(), "localhost".to_string());
        config.settings.insert("port".to_string(), "8080".to_string());

        let result = config.validate_required(&["host", "port"]);
        assert!(result.is_ok());

        let result = config.validate_required(&["host", "port", "missing_key"]);
        assert!(result.is_err());
        if let Err(missing) = result {
            assert_eq!(missing, vec!["missing_key".to_string()]);
        }
    }
}