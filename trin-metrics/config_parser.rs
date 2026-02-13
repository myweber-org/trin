use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub cache_size: usize,
    pub features: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database_url: String::from("postgresql://localhost:5432/mydb"),
            port: 8080,
            log_level: String::from("info"),
            cache_size: 100,
            features: vec![String::from("api"), String::from("auth")],
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config = Config::default();
        let parsed: HashMap<String, String> = content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect();
        
        if let Some(url) = parsed.get("DATABASE_URL") {
            if !url.starts_with("postgresql://") {
                return Err("Invalid database URL format".to_string());
            }
            config.database_url = url.clone();
        }
        
        if let Some(port_str) = parsed.get("PORT") {
            config.port = port_str.parse()
                .map_err(|_| "PORT must be a valid number".to_string())?;
            if config.port == 0 {
                return Err("PORT cannot be zero".to_string());
            }
        }
        
        if let Some(level) = parsed.get("LOG_LEVEL") {
            let valid_levels = ["error", "warn", "info", "debug", "trace"];
            if !valid_levels.contains(&level.as_str()) {
                return Err(format!("LOG_LEVEL must be one of: {}", valid_levels.join(", ")));
            }
            config.log_level = level.clone();
        }
        
        if let Some(size_str) = parsed.get("CACHE_SIZE") {
            config.cache_size = size_str.parse()
                .map_err(|_| "CACHE_SIZE must be a valid number".to_string())?;
        }
        
        if let Some(features_str) = parsed.get("FEATURES") {
            config.features = features_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.database_url.is_empty() {
            errors.push("DATABASE_URL cannot be empty".to_string());
        }
        
        if self.port > 65535 {
            errors.push("PORT must be between 1 and 65535".to_string());
        }
        
        if self.cache_size == 0 {
            errors.push("CACHE_SIZE must be greater than zero".to_string());
        }
        
        if self.features.is_empty() {
            errors.push("At least one feature must be enabled".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.features.contains(&"api".to_string()));
    }
    
    #[test]
    fn test_valid_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgresql://localhost:5432/testdb").unwrap();
        writeln!(file, "PORT=3000").unwrap();
        writeln!(file, "LOG_LEVEL=debug").unwrap();
        writeln!(file, "CACHE_SIZE=500").unwrap();
        writeln!(file, "FEATURES=api,auth,cache").unwrap();
        
        let config = Config::from_file(file.path()).unwrap();
        assert_eq!(config.database_url, "postgresql://localhost:5432/testdb");
        assert_eq!(config.port, 3000);
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_size, 500);
        assert_eq!(config.features.len(), 3);
    }
    
    #[test]
    fn test_invalid_port() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PORT=0").unwrap();
        
        let result = Config::from_file(file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("PORT cannot be zero"));
    }
    
    #[test]
    fn test_validation_success() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_validation_failure() {
        let mut config = Config::default();
        config.port = 70000;
        config.features.clear();
        
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }
}use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_file_size_mb: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                max_connections: 20,
                min_connections: 5,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "./logs/app.log".to_string(),
                max_file_size_mb: 100,
            },
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_file_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}, using defaults", e);
                Self::default()
            }
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        if self.database.max_connections < self.database.min_connections {
            return Err("Max connections must be greater than or equal to min connections".to_string());
        }
        if self.logging.max_file_size_mb == 0 {
            return Err("Max file size must be greater than 0".to_string());
        }
        Ok(())
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let toml_content = self.to_toml()?;
        fs::write(path, toml_content)?;
        Ok(())
    }
}