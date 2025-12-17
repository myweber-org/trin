use std::collections::HashMap;
use std::fs;
use std::io;

#[derive(Debug)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, io::Error> {
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

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> &str {
        self.get(key).unwrap_or(default)
    }

    pub fn get_parsed<T>(&self, key: &str) -> Result<T, String>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        self.get(key)
            .ok_or_else(|| format!("Key '{}' not found", key))?
            .parse()
            .map_err(|e| format!("Failed to parse '{}': {}", key, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Sample config").unwrap();
        writeln!(file, "host=localhost").unwrap();
        writeln!(file, "port=8080").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "timeout=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("host"), Some("localhost"));
        assert_eq!(config.get("port"), Some("8080"));
        assert_eq!(config.get("timeout"), Some("30"));
        assert_eq!(config.get("missing"), None);
    }

    #[test]
    fn test_get_or_default() {
        let config = Config::new();
        assert_eq!(config.get_or_default("missing", "default"), "default");
    }

    #[test]
    fn test_get_parsed() {
        let mut config = Config::new();
        config.set("port", "8080");
        config.set("enabled", "true");

        let port: u16 = config.get_parsed("port").unwrap();
        let enabled: bool = config.get_parsed("enabled").unwrap();

        assert_eq!(port, 8080);
        assert_eq!(enabled, true);
    }
}