use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        let mut settings = HashMap::new();
        settings.insert("default_port".to_string(), "8080".to_string());
        settings.insert("log_level".to_string(), "info".to_string());
        Config { settings }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut settings = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                settings.insert(key, value);
            }
        }

        Ok(Config { settings })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        if let Ok(env_value) = env::var(key) {
            return Some(&env_value);
        }
        self.settings.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.settings.insert(key.to_string(), value.to_string());
    }

    pub fn load_with_fallback(&mut self, path: &str) {
        if let Ok(file_config) = Self::from_file(path) {
            for (key, value) in file_config.settings {
                self.settings.insert(key, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_creation() {
        let config = Config::new();
        assert_eq!(config.get("default_port"), Some(&"8080".to_string()));
        assert_eq!(config.get("log_level"), Some(&"info".to_string()));
    }

    #[test]
    fn test_config_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "server_host=localhost").unwrap();
        writeln!(temp_file, "server_port=3000").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "timeout=30").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("server_host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("server_port"), Some(&"3000".to_string()));
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.get("nonexistent"), None);
    }

    #[test]
    fn test_environment_override() {
        env::set_var("CUSTOM_KEY", "env_value");
        let config = Config::new();
        config.set("CUSTOM_KEY", "file_value");
        
        assert_eq!(config.get("CUSTOM_KEY"), Some(&"env_value".to_string()));
        env::remove_var("CUSTOM_KEY");
    }
}