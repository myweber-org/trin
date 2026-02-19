use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut values = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "ENVIRONMENT=production").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("ENVIRONMENT"), Some(&"production".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=$DB_PASSWORD").unwrap();
        writeln!(file, "NORMAL=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NORMAL"), Some(&"value".to_string()));
    }
}use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub enable_ssl: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 100,
            enable_ssl: false,
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let mut config: ServerConfig = toml::from_str(&config_str)?;
    
    validate_config(&mut config)?;
    
    Ok(config)
}

fn validate_config(config: &mut ServerConfig) -> Result<(), String> {
    if config.port == 0 {
        return Err("Port cannot be zero".to_string());
    }
    
    if config.max_connections == 0 {
        config.max_connections = ServerConfig::default().max_connections;
    }
    
    if config.host.is_empty() {
        config.host = ServerConfig::default().host;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_valid_config() {
        let config_toml = r#"
            host = "0.0.0.0"
            port = 3000
            max_connections = 500
            enable_ssl = true
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), config_toml).unwrap();
        
        let config = load_config(file.path()).unwrap();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert_eq!(config.max_connections, 500);
        assert_eq!(config.enable_ssl, true);
    }
    
    #[test]
    fn test_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.enable_ssl, false);
    }
}