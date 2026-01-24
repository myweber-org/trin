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
                let processed_value = Self::process_value(value.trim());
                values.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map_or(default.to_string(), |v| v.clone())
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
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=${DB_PASSWORD}").unwrap();
        writeln!(file, "HOST=localhost").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub use_ssl: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub features: HashMap<String, bool>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_connections: 100,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                pool_size: 10,
                use_ssl: false,
            },
            features: HashMap::from([
                ("logging".to_string(), true),
                ("metrics".to_string(), false),
                ("caching".to_string(), true),
            ]),
        }
    }
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        
        config.validate()?;
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        
        if self.server.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }
        
        if self.database.pool_size == 0 {
            return Err("Database pool size must be greater than 0".to_string());
        }
        
        Ok(())
    }

    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}