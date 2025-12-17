use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub defaults: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            defaults: HashMap::from([
                ("timeout".to_string(), "30".to_string()),
                ("retries".to_string(), "3".to_string()),
                ("log_level".to_string(), "info".to_string()),
            ]),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            if value.is_empty() {
                return Err(format!("Empty value for key: {}", key));
            }

            self.settings.insert(key, value);
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key).or_else(|| self.defaults.get(key))
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }

    pub fn validate_required(&self, keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();

        for key in keys {
            if !self.settings.contains_key(*key) && !self.defaults.contains_key(*key) {
                missing.push(format!("Required key '{}' not found", key));
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    pub fn merge(&mut self, other: Config) {
        for (key, value) in other.settings {
            self.settings.insert(key, value);
        }
        for (key, value) in other.defaults {
            self.defaults.entry(key).or_insert(value);
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
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "host=localhost\nport=8080\n").unwrap();

        let result = config.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_default_values() {
        let config = Config::new();
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.get("log_level"), Some(&"info".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_validation() {
        let config = Config::new();
        let result = config.validate_required(&["timeout", "host"]);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("host")));
    }
}