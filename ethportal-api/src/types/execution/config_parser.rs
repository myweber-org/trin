use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub thresholds: HashMap<String, f64>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut settings = HashMap::new();
        let mut thresholds = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').map(|s| s.trim()).collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].to_string();
            let value = parts[1].to_string();

            if key.starts_with("threshold_") {
                let num_value: f64 = value.parse()
                    .map_err(|_| format!("Invalid number for key {}: {}", key, value))?;
                thresholds.insert(key, num_value);
            } else {
                settings.insert(key, value);
            }
        }

        Ok(Config { settings, thresholds })
    }

    pub fn get_setting(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_threshold(&self, key: &str) -> Option<f64> {
        self.thresholds.get(key).copied()
    }
}use std::fs;
use std::collections::HashMap;
use std::error::Error;

pub type ConfigMap = HashMap<String, String>;

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError(String),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

pub fn parse_config_file(path: &str) -> Result<ConfigMap, ConfigError> {
    let content = fs::read_to_string(path)?;
    let mut config = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(ConfigError::ParseError(
                format!("Invalid format at line {}", line_num + 1)
            ));
        }

        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();
        
        if key.is_empty() {
            return Err(ConfigError::ParseError(
                format!("Empty key at line {}", line_num + 1)
            ));
        }

        config.insert(key, value);
    }

    Ok(config)
}

pub fn get_config_value(config: &ConfigMap, key: &str) -> Option<&String> {
    config.get(key)
}