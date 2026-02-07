use std::collections::HashMap;
use std::env;
use std::fs;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut settings = HashMap::new();
        let var_regex = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = line.split_once('=') {
                let key = key.trim().to_string();
                value = value.trim();

                let mut processed_value = value.to_string();
                for cap in var_regex.captures_iter(value) {
                    if let Some(var_name) = cap.get(1) {
                        let var_name = var_name.as_str();
                        if let Ok(env_value) = env::var(var_name) {
                            processed_value = processed_value.replace(&cap[0], &env_value);
                        }
                    }
                }

                settings.insert(key, processed_value);
            }
        }

        Ok(Config { settings })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.settings.get(key).cloned().unwrap_or(default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
            database_url=postgres://localhost:5432/mydb
            max_connections=10
            # This is a comment
            debug_mode=true
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("database_url").unwrap(), "postgres://localhost:5432/mydb");
        assert_eq!(config.get("max_connections").unwrap(), "10");
        assert_eq!(config.get("debug_mode").unwrap(), "true");
        assert!(config.get("nonexistent").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "localhost");
        env::set_var("DB_PORT", "5432");

        let content = r#"
            database_url=postgres://${DB_HOST}:${DB_PORT}/mydb
            api_key=${SECRET_KEY}
        "#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("database_url").unwrap(), "postgres://localhost:5432/mydb");
        assert_eq!(config.get("api_key").unwrap(), "");
    }

    #[test]
    fn test_file_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "server_host=127.0.0.1").unwrap();
        writeln!(temp_file, "server_port=8080").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("server_host").unwrap(), "127.0.0.1");
        assert_eq!(config.get("server_port").unwrap(), "8080");
    }

    #[test]
    fn test_get_or_default() {
        let content = "existing_key=value";
        let config = Config::from_str(content).unwrap();
        
        assert_eq!(config.get_or_default("existing_key", "default"), "value");
        assert_eq!(config.get_or_default("missing_key", "default_value"), "default_value");
    }
}use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub debug_mode: bool,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        if self.database.port == 0 {
            return Err("Database port cannot be zero".to_string());
        }
        if self.server.max_connections == 0 {
            return Err("Max connections must be greater than zero".to_string());
        }
        if !["error", "warn", "info", "debug", "trace"].contains(&self.log_level.as_str()) {
            return Err("Invalid log level specified".to_string());
        }
        Ok(())
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database_name
        )
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.address, self.server.port)
    }
}