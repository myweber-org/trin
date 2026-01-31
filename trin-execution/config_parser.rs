
use std::collections::HashMap;
use std::env;
use std::fs;
use regex::Regex;

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut settings = HashMap::new();
        let var_regex = Regex::new(r"\$\{([A-Za-z0-9_]+)\}").unwrap();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, mut value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                
                for cap in var_regex.captures_iter(&value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            value = value.replace(&cap[0], &env_value);
                        }
                    }
                }
                
                settings.insert(key, value.trim().to_string());
            }
        }
        
        Ok(Config { settings })
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
}