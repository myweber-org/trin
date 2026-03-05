use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config = Config::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                config.set(key.trim(), value.trim());
            }
        }

        Ok(config)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_with_env_fallback(&self, key: &str) -> Option<String> {
        if let Some(value) = self.get(key) {
            return Some(value.clone());
        }

        env::var(key).ok()
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.get_with_env_fallback(key)
            .unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/test");
        assert_eq!(config.get("API_KEY").unwrap(), "secret123");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_fallback() {
        env::set_var("TEST_ENV_VAR", "env_value");
        let config = Config::new();
        let value = config.get_with_env_fallback("TEST_ENV_VAR");
        assert_eq!(value.unwrap(), "env_value");
        env::remove_var("TEST_ENV_VAR");
    }

    #[test]
    fn test_default_value() {
        let config = Config::new();
        let value = config.get_or_default("MISSING_KEY", "default_value");
        assert_eq!(value, "default_value");
    }
}