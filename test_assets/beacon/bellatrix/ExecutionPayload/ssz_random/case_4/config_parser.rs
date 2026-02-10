use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub thresholds: HashMap<String, f64>,
    pub enabled: bool,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        Self::parse(&content)
    }

    fn parse(content: &str) -> Result<Self, String> {
        let mut settings = HashMap::new();
        let mut thresholds = HashMap::new();
        let mut enabled = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid line format: {}", line));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            match key.as_str() {
                "enabled" => {
                    enabled = value.parse()
                        .map_err(|_| format!("Invalid boolean value for 'enabled': {}", value))?;
                }
                key if key.starts_with("threshold_") => {
                    let threshold_value: f64 = value.parse()
                        .map_err(|_| format!("Invalid float value for '{}': {}", key, value))?;
                    thresholds.insert(key, threshold_value);
                }
                _ => {
                    settings.insert(key, value);
                }
            }
        }

        Ok(Config {
            settings,
            thresholds,
            enabled,
        })
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_threshold(&self, key: &str) -> Option<&f64> {
        self.thresholds.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let content = r#"
            enabled = true
            server_host = 127.0.0.1
            server_port = 8080
            threshold_cpu = 80.5
            threshold_memory = 90.0
        "#;

        let config = Config::parse(content).unwrap();
        assert_eq!(config.enabled, true);
        assert_eq!(config.get_setting("server_host"), Some(&"127.0.0.1".to_string()));
        assert_eq!(config.get_setting("server_port"), Some(&"8080".to_string()));
        assert_eq!(config.get_threshold("threshold_cpu"), Some(&80.5));
        assert_eq!(config.get_threshold("threshold_memory"), Some(&90.0));
    }

    #[test]
    fn test_parse_invalid_boolean() {
        let content = "enabled = not_a_bool";
        let result = Config::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_float() {
        let content = "threshold_cpu = not_a_number";
        let result = Config::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_config() {
        let content = "";
        let config = Config::parse(content).unwrap();
        assert_eq!(config.enabled, false);
        assert!(config.settings.is_empty());
        assert!(config.thresholds.is_empty());
    }
}