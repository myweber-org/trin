use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub cache_ttl: u64,
    pub feature_flags: HashMap<String, bool>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingVariable("DATABASE_URL".to_string()))?;
        
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidValue("SERVER_PORT".to_string()))?;
        
        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());
        
        let cache_ttl = env::var("CACHE_TTL")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .map_err(|_| ConfigError::InvalidValue("CACHE_TTL".to_string()))?;
        
        let feature_flags = Self::parse_feature_flags();
        
        Ok(Self {
            database_url,
            server_port,
            log_level,
            cache_ttl,
            feature_flags,
        })
    }
    
    fn parse_feature_flags() -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        
        for (key, value) in env::vars() {
            if key.starts_with("FEATURE_") {
                let flag_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = value.to_lowercase() == "true" || value == "1";
                flags.insert(flag_name, enabled);
            }
        }
        
        flags
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server_port == 0 {
            return Err(ConfigError::InvalidValue("SERVER_PORT cannot be zero".to_string()));
        }
        
        if self.database_url.is_empty() {
            return Err(ConfigError::MissingVariable("DATABASE_URL".to_string()));
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingVariable(String),
    InvalidValue(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingVariable(var) => write!(f, "Missing environment variable: {}", var),
            ConfigError::InvalidValue(msg) => write!(f, "Invalid configuration value: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}