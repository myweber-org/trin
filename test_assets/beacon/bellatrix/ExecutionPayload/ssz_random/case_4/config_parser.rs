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
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut values = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let processed_value = Self::interpolate_env_vars(value.trim());
                values.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { values })
    }

    fn interpolate_env_vars(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next();
                let mut var_name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '}' {
                        chars.next();
                        break;
                    }
                    var_name.push(ch);
                    chars.next();
                }
                
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
                } else {
                    result.push_str(&format!("${{{}}}", var_name));
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "APP_NAME=MyApp").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "VERSION=1.0.0").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("APP_NAME"), Some(&"MyApp".to_string()));
        assert_eq!(config.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_interpolation() {
        env::set_var("DB_HOST", "localhost");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://${DB_HOST}:5432").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some(&"postgres://localhost:5432".to_string()));
    }
}