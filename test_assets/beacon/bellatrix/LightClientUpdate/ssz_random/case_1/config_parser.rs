use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        let re = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+?)\s*$").unwrap();
        
        for line in content.lines() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            
            if let Some(caps) = re.captures(line) {
                let key = caps[1].to_string();
                let mut value = caps[2].to_string();
                
                self.process_environment_variables(&mut value);
                self.values.insert(key, value);
            } else {
                return Err(format!("Invalid configuration line: {}", line));
            }
        }
        
        Ok(())
    }
    
    fn process_environment_variables(&self, value: &mut String) {
        let env_regex = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
        
        *value = env_regex.replace_all(value, |caps: &regex::Captures| {
            let var_name = &caps[1];
            env::var(var_name).unwrap_or_else(|_| String::new())
        }).to_string();
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_parsing() {
        let mut parser = ConfigParser::new();
        let config = r#"
            DATABASE_HOST=localhost
            DATABASE_PORT=5432
            # This is a comment
            API_KEY=secret123
        "#;
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(parser.get("API_KEY"), Some(&"secret123".to_string()));
    }
    
    #[test]
    fn test_environment_substitution() {
        env::set_var("APP_ENV", "production");
        
        let mut parser = ConfigParser::new();
        let config = r#"ENVIRONMENT=${APP_ENV}"#;
        
        assert!(parser.load_from_str(config).is_ok());
        assert_eq!(parser.get("ENVIRONMENT"), Some(&"production".to_string()));
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

pub fn load_config(config_path: &str) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let path = Path::new(config_path);
    
    if !path.exists() {
        let default_config = AppConfig::default();
        let config_str = toml::to_string_pretty(&default_config)?;
        fs::write(config_path, config_str)?;
        return Ok(default_config);
    }
    
    let config_content = fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&config_content)?;
    
    validate_config(&config)?;
    
    Ok(config)
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }
    
    if config.database.max_connections < config.database.min_connections {
        return Err("Max connections must be greater than or equal to min connections".to_string());
    }
    
    if config.logging.max_file_size_mb == 0 {
        return Err("Max file size must be greater than 0".to_string());
    }
    
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        return Err(format!("Invalid log level: {}", config.logging.level));
    }
    
    Ok(())
}

pub fn save_config(config: &AppConfig, config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    validate_config(config)?;
    
    let config_str = toml::to_string_pretty(config)?;
    fs::write(config_path, config_str)?;
    
    Ok(())
}