
use std::fs;
use std::collections::HashMap;
use std::error::Error;

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

    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
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
            }
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.settings.insert(key.to_string(), value.to_string());
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
        writeln!(temp_file, "# Sample config").unwrap();
        writeln!(temp_file, "host = localhost").unwrap();
        writeln!(temp_file, "port = 8080").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "debug = true").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("debug"), Some(&"true".to_string()));
        assert_eq!(config.get("missing"), None);
    }

    #[test]
    fn test_config_operations() {
        let mut config = Config::new();
        config.set("timeout", "30");
        config.set("retries", "3");

        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.get("retries"), Some(&"3".to_string()));
    }
}