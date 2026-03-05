use std::env;
use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
    pub feature_flags: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config_map = HashMap::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = trimmed.split_once('=') {
                config_map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        
        Self::from_map(config_map)
    }
    
    fn from_map(mut map: HashMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = Self::get_value(&mut map, "DATABASE_URL")?;
        let api_key = Self::get_value(&mut map, "API_KEY")?;
        let debug_mode = Self::get_bool(&mut map, "DEBUG_MODE")?;
        let port = Self::get_u16(&mut map, "PORT")?;
        
        let mut feature_flags = HashMap::new();
        for (key, value) in map {
            if key.starts_with("FEATURE_") {
                let flag_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let flag_value = value.parse::<bool>().unwrap_or(false);
                feature_flags.insert(flag_name, flag_value);
            }
        }
        
        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
            feature_flags,
        })
    }
    
    fn get_value(map: &mut HashMap<String, String>, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(env_value) = env::var(key).ok() {
            return Ok(env_value);
        }
        
        map.remove(key)
            .ok_or_else(|| format!("Missing configuration: {}", key).into())
    }
    
    fn get_bool(map: &mut HashMap<String, String>, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let value = Self::get_value(map, key)?;
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(format!("Invalid boolean value for {}: {}", key, value).into()),
        }
    }
    
    fn get_u16(map: &mut HashMap<String, String>, key: &str) -> Result<u16, Box<dyn std::error::Error>> {
        let value = Self::get_value(map, key)?;
        value.parse::<u16>()
            .map_err(|e| format!("Invalid port number for {}: {}", key, e).into())
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.feature_flags.get(feature).copied().unwrap_or(false)
    }
}