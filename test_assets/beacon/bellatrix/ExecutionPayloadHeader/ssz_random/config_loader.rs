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
        let content = fs::read_to_string(path)?;
        let mut config = Config::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                match key {
                    "DATABASE_URL" => config.database_url = Self::resolve_env_var(value),
                    "SERVER_PORT" => {
                        if let Ok(port) = value.parse() {
                            config.server_port = port;
                        }
                    }
                    "LOG_LEVEL" => config.log_level = value.to_string(),
                    _ if key.starts_with("FEATURE_") => {
                        let feature_name = key.trim_start_matches("FEATURE_").to_lowercase();
                        let enabled = value.to_lowercase() == "true" || value == "1";
                        config.features.insert(feature_name, enabled);
                    }
                    _ => {}
                }
            }
        }
        
        Ok(config)
    }
    
    fn resolve_env_var(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/db".to_string(),
            server_port: 8080,
            log_level: "info".to_string(),
            features: HashMap::new(),
        }
    }
}