use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut values = HashMap::new();
        let var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                
                for cap in var_pattern.captures_iter(&value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            value = value.replace(&cap[0], &env_value);
                        }
                    }
                }
                
                values.insert(key, value.trim().to_string());
            }
        }

        Ok(Config { values })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let content = "host=localhost\nport=8080\n# comment\n\ndebug=true";
        let config = Config::from_str(content).unwrap();
        
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("debug"), Some(&"true".to_string()));
        assert_eq!(config.get("missing"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "9090");
        
        let content = "host=localhost\nport=${APP_PORT}\nurl=http://${host}:${APP_PORT}";
        let config = Config::from_str(content).unwrap();
        
        assert_eq!(config.get("port"), Some(&"9090".to_string()));
        assert_eq!(config.get("url"), Some(&"http://localhost:9090".to_string()));
    }

    #[test]
    fn test_file_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "database=postgresql").unwrap();
        writeln!(file, "timeout=30").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("database"), Some(&"postgresql".to_string()));
        assert_eq!(config.get_or_default("timeout", "10"), "30");
        assert_eq!(config.get_or_default("missing", "default_value"), "default_value");
    }
}use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&config_str)?;
        
        config.apply_environment_overrides();
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) {
        if let Ok(host) = env::var("SERVER_HOST") {
            self.server.host = host;
        }
        
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                self.server.port = port_num;
            }
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database.url = db_url;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.logging.level = log_level;
        }
    }
    
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".into());
        }
        
        if self.server.max_connections == 0 {
            return Err("Max connections must be greater than 0".into());
        }
        
        if self.database.pool_size == 0 {
            return Err("Database pool size must be greater than 0".into());
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.logging.level.to_lowercase().as_str()) {
            return Err(format!("Invalid log level: {}", self.logging.level).into());
        }
        
        Ok(())
    }
    
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let config_content = r#"
            [server]
            host = "127.0.0.1"
            port = 8080
            max_connections = 100
            
            [database]
            url = "postgresql://localhost/mydb"
            pool_size = 10
            timeout_seconds = 30
            
            [logging]
            level = "info"
            file_path = "/var/log/app.log"
            enable_console = true
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, config_content.as_bytes()).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.pool_size, 10);
        assert_eq!(config.logging.level, "info");
    }
    
    #[test]
    fn test_environment_overrides() {
        env::set_var("SERVER_HOST", "0.0.0.0");
        env::set_var("LOG_LEVEL", "debug");
        
        let config_content = r#"
            [server]
            host = "127.0.0.1"
            port = 8080
            max_connections = 100
            
            [database]
            url = "postgresql://localhost/mydb"
            pool_size = 10
            timeout_seconds = 30
            
            [logging]
            level = "info"
            enable_console = true
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, config_content.as_bytes()).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.logging.level, "debug");
        
        env::remove_var("SERVER_HOST");
        env::remove_var("LOG_LEVEL");
    }
    
    #[test]
    fn test_validation_failure() {
        let config_content = r#"
            [server]
            host = "127.0.0.1"
            port = 0
            max_connections = 100
            
            [database]
            url = "postgresql://localhost/mydb"
            pool_size = 10
            timeout_seconds = 30
            
            [logging]
            level = "info"
            enable_console = true
        "#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, config_content.as_bytes()).unwrap();
        
        let result = AppConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }
}