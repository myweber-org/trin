use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let mut config = Self::default();
        
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                config.parse_key_value(key, value);
            }
        }
        
        config.apply_environment_overrides();
        Ok(config)
    }
    
    fn parse_key_value(&mut self, key: &str, value: &str) {
        match key {
            "DATABASE_URL" => self.database_url = value.to_string(),
            "SERVER_PORT" => {
                if let Ok(port) = value.parse() {
                    self.server_port = port;
                }
            }
            "LOG_LEVEL" => self.log_level = value.to_string(),
            _ if key.starts_with("FEATURE_") => {
                let feature_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = value.to_lowercase() == "true" || value == "1";
                self.features.insert(feature_name, enabled);
            }
            _ => {}
        }
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                self.server_port = port_num;
            }
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
        
        for (key, value) in env::vars() {
            if key.starts_with("FEATURE_") {
                let feature_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = value.to_lowercase() == "true" || value == "1";
                self.features.insert(feature_name, enabled);
            }
        }
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: String::from("postgresql://localhost:5432/mydb"),
            server_port: 8080,
            log_level: String::from("info"),
            features: HashMap::new(),
        }
    }
}