use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&config_str)?;
        
        config.apply_environment_overrides();
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                self.server_port = port_num;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database_url = db_url;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
        
        if let Ok(cache_ttl) = env::var("CACHE_TTL") {
            if let Ok(ttl) = cache_ttl.parse() {
                self.cache_ttl = ttl;
            }
        }
    }
    
    pub fn default() -> Self {
        Self {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/app_db"),
            log_level: String::from("info"),
            cache_ttl: 3600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
            server_port = 3000
            database_url = "postgresql://localhost:5432/test_db"
            log_level = "debug"
            cache_ttl = 1800
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.database_url, "postgresql://localhost:5432/test_db");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_ttl, 1800);
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("SERVER_PORT", "9090");
        env::set_var("LOG_LEVEL", "trace");
        
        let toml_content = r#"
            server_port = 3000
            database_url = "postgresql://localhost:5432/test_db"
            log_level = "debug"
            cache_ttl = 1800
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_port, 9090);
        assert_eq!(config.log_level, "trace");
        
        env::remove_var("SERVER_PORT");
        env::remove_var("LOG_LEVEL");
    }
}