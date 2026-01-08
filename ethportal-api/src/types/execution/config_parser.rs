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
}