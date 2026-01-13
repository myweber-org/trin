
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
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.json".to_string());
        
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
        
        let mut config: HashMap<String, serde_json::Value> = serde_json::from_str(&config_content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                config.insert(config_key, serde_json::Value::String(value));
            }
        }
        
        let database_url = config
            .get("database_url")
            .and_then(|v| v.as_str())
            .ok_or("Missing database_url in config")?
            .to_string();
        
        let server_port = config
            .get("server_port")
            .and_then(|v| v.as_u64())
            .map(|p| p as u16)
            .unwrap_or(8080);
        
        let log_level = config
            .get("log_level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_string();
        
        let features = config
            .get("features")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_bool().map(|b| (k.clone(), b)))
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(Config {
            database_url,
            server_port,
            log_level,
            features,
        })
    }
    
    pub fn is_feature_enabled(&self, feature_name: &str) -> bool {
        self.features.get(feature_name).copied().unwrap_or(false)
    }
}