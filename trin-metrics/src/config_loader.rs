use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
        
        let mut config: HashMap<String, String> = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        
        // Override with environment variables
        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.insert("database_url".to_string(), db_url);
        }
        
        if let Ok(api_key) = env::var("API_KEY") {
            config.insert("api_key".to_string(), api_key);
        }
        
        if let Ok(debug) = env::var("DEBUG_MODE") {
            config.insert("debug_mode".to_string(), debug);
        }
        
        if let Ok(port) = env::var("PORT") {
            config.insert("port".to_string(), port);
        }
        
        let database_url = config.get("database_url")
            .ok_or("Missing database_url in config")?
            .clone();
        
        let api_key = config.get("api_key")
            .ok_or("Missing api_key in config")?
            .clone();
        
        let debug_mode = config.get("debug_mode")
            .map(|s| s.parse().unwrap_or(false))
            .unwrap_or(false);
        
        let port = config.get("port")
            .map(|s| s.parse().unwrap_or(8080))
            .unwrap_or(8080);
        
        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
        })
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        
        if self.api_key.is_empty() {
            return Err("API key cannot be empty".to_string());
        }
        
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_loading() {
        let mut file = NamedTempFile::new().unwrap();
        let config_content = r#"
            database_url = "postgres://localhost/test"
            api_key = "test_key"
            debug_mode = "true"
            port = "3000"
        "#;
        
        fs::write(file.path(), config_content).unwrap();
        
        env::set_var("CONFIG_PATH", file.path().to_str().unwrap());
        
        let config = Config::load().unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.port, 3000);
    }
    
    #[test]
    fn test_env_override() {
        let mut file = NamedTempFile::new().unwrap();
        let config_content = r#"
            database_url = "original_url"
            api_key = "original_key"
        "#;
        
        fs::write(file.path(), config_content).unwrap();
        
        env::set_var("CONFIG_PATH", file.path().to_str().unwrap());
        env::set_var("DATABASE_URL", "overridden_url");
        
        let config = Config::load().unwrap();
        assert_eq!(config.database_url, "overridden_url");
        assert_eq!(config.api_key, "original_key");
        
        env::remove_var("DATABASE_URL");
    }
}