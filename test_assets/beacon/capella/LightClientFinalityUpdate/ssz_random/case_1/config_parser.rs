
use serde::{Deserialize, Serialize};
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
    pub enable_tls: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_file_size_mb: u64,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = toml::to_string_pretty(self)?;
        fs::write(path, config_str)?;
        Ok(())
    }

    pub fn default_config() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                enable_tls: false,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/mydb".to_string(),
                max_connections: 10,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "logs/app.log".to_string(),
                max_file_size_mb: 100,
            },
        }
    }
}

pub fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be zero".to_string());
    }
    if config.database.max_connections == 0 {
        return Err("Database max connections cannot be zero".to_string());
    }
    if config.logging.max_file_size_mb == 0 {
        return Err("Log file max size cannot be zero".to_string());
    }
    Ok(())
}use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    ParseError(String),
    DuplicateKey(String),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

pub type ConfigMap = HashMap<String, String>;

pub fn parse_config_file<P: AsRef<Path>>(path: P) -> Result<ConfigMap, ConfigError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut config = HashMap::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.splitn(2, '=').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(ConfigError::ParseError(format!(
                "Invalid format at line {}: '{}'",
                line_num + 1,
                trimmed
            )));
        }

        let key = parts[0].to_string();
        let value = parts[1].to_string();

        if config.contains_key(&key) {
            return Err(ConfigError::DuplicateKey(key));
        }

        config.insert(key, value);
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "host=localhost").unwrap();
        writeln!(temp_file, "port=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "timeout=30").unwrap();

        let config = parse_config_file(temp_file.path()).unwrap();
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
        assert_eq!(config.len(), 3);
    }

    #[test]
    fn test_duplicate_key() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "key=value1").unwrap();
        writeln!(temp_file, "key=value2").unwrap();

        let result = parse_config_file(temp_file.path());
        assert!(matches!(result, Err(ConfigError::DuplicateKey(_))));
    }

    #[test]
    fn test_invalid_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "key_without_value").unwrap();

        let result = parse_config_file(temp_file.path());
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
    }
}