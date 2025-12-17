use std::fs;
use std::collections::HashMap;

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

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config = Config::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                config.settings.insert(key, value);
            } else {
                return Err(format!("Invalid config line: {}", line));
            }
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
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
        writeln!(temp_file, "host=localhost\nport=8080\n# This is a comment").unwrap();

        let config = Config::load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_load_invalid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid_line_without_equals").unwrap();

        let result = Config::load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}