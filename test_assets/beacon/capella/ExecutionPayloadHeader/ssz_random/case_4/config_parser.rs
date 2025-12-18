
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let parsed = parse_config(&content)?;
        
        Ok(Config {
            database_url: get_value(&parsed, "DATABASE_URL")?,
            api_key: get_value(&parsed, "API_KEY")?,
            debug_mode: get_value(&parsed, "DEBUG")?.parse()?,
            port: get_value(&parsed, "PORT")?.parse()?,
        })
    }
}

fn parse_config(content: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut config_map = HashMap::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let processed_value = process_value(value.trim());
            config_map.insert(key, processed_value);
        }
    }
    
    Ok(config_map)
}

fn process_value(value: &str) -> String {
    if value.starts_with('$') {
        let var_name = &value[1..];
        env::var(var_name).unwrap_or_else(|_| value.to_string())
    } else {
        value.to_string()
    }
}

fn get_value(map: &HashMap<String, String>, key: &str) -> Result<String, Box<dyn std::error::Error>> {
    map.get(key)
        .cloned()
        .ok_or_else(|| format!("Missing configuration key: {}", key).into())
}