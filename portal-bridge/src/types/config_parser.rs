
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut settings = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let processed_value = Self::substitute_env_vars(value.trim());
                settings.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { settings })
    }

    fn substitute_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        for (key, env_value) in env::vars() {
            let placeholder = format!("${}", key);
            result = result.replace(&placeholder, &env_value);
        }
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost:5432").unwrap();
        writeln!(file, "API_KEY=${SECRET_KEY}").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        env::set_var("SECRET_KEY", "abc123");

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(config.get("API_KEY").unwrap(), "abc123");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
        assert!(config.get("NONEXISTENT").is_none());
    }
}use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub pool_timeout_seconds: u32,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                max_connections: 10,
                pool_timeout_seconds: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                enable_console: true,
            },
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|_| ConfigError::FileNotFound(path.to_string()))?;

        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        let parsed: HashMap<String, toml::Value> = toml::from_str(content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        let mut config = Config::default();

        if let Some(server_table) = parsed.get("server").and_then(|v| v.as_table()) {
            if let Some(host) = server_table.get("host").and_then(|v| v.as_str()) {
                config.server.host = host.to_string();
            }
            if let Some(port) = server_table.get("port").and_then(|v| v.as_integer()) {
                if port < 1 || port > 65535 {
                    return Err(ConfigError::ValidationError(
                        "Port must be between 1 and 65535".to_string(),
                    ));
                }
                config.server.port = port as u16;
            }
            if let Some(timeout) = server_table.get("timeout_seconds").and_then(|v| v.as_integer()) {
                if timeout < 1 {
                    return Err(ConfigError::ValidationError(
                        "Timeout must be positive".to_string(),
                    ));
                }
                config.server.timeout_seconds = timeout as u64;
            }
        }

        if let Some(db_table) = parsed.get("database").and_then(|v| v.as_table()) {
            if let Some(url) = db_table.get("url").and_then(|v| v.as_str()) {
                config.database.url = url.to_string();
            }
            if let Some(max_conn) = db_table.get("max_connections").and_then(|v| v.as_integer()) {
                if max_conn < 1 {
                    return Err(ConfigError::ValidationError(
                        "Max connections must be positive".to_string(),
                    ));
                }
                config.database.max_connections = max_conn as u32;
            }
            if let Some(pool_timeout) = db_table.get("pool_timeout_seconds").and_then(|v| v.as_integer()) {
                if pool_timeout < 1 {
                    return Err(ConfigError::ValidationError(
                        "Pool timeout must be positive".to_string(),
                    ));
                }
                config.database.pool_timeout_seconds = pool_timeout as u32;
            }
        }

        if let Some(log_table) = parsed.get("logging").and_then(|v| v.as_table()) {
            if let Some(level) = log_table.get("level").and_then(|v| v.as_str()) {
                let valid_levels = ["error", "warn", "info", "debug", "trace"];
                if !valid_levels.contains(&level.to_lowercase().as_str()) {
                    return Err(ConfigError::ValidationError(format!(
                        "Invalid log level '{}'. Must be one of: {:?}",
                        level, valid_levels
                    )));
                }
                config.logging.level = level.to_string();
            }
            if let Some(file_path) = log_table.get("file_path").and_then(|v| v.as_str()) {
                config.logging.file_path = Some(file_path.to_string());
            }
            if let Some(enable_console) = log_table.get("enable_console").and_then(|v| v.as_bool()) {
                config.logging.enable_console = enable_console;
            }
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.host.is_empty() {
            return Err(ConfigError::ValidationError(
                "Server host cannot be empty".to_string(),
            ));
        }

        if self.database.url.is_empty() {
            return Err(ConfigError::ValidationError(
                "Database URL cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_valid_config_parsing() {
        let toml_content = r#"
            [server]
            host = "0.0.0.0"
            port = 9000
            timeout_seconds = 60

            [database]
            url = "postgresql://prod:5432/appdb"
            max_connections = 20
            pool_timeout_seconds = 30

            [logging]
            level = "debug"
            file_path = "/var/log/app.log"
            enable_console = false
        "#;

        let config = Config::from_str(toml_content).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.logging.file_path, Some("/var/log/app.log".to_string()));
        assert!(!config.logging.enable_console);
    }

    #[test]
    fn test_invalid_log_level() {
        let toml_content = r#"
            [logging]
            level = "invalid"
        "#;

        let result = Config::from_str(toml_content);
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));
    }

    #[test]
    fn test_validation() {
        let mut config = Config::default();
        config.database.url = String::new();
        
        let result = config.validate();
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));
    }
}