use std::fs;
use std::collections::HashMap;
use toml;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub pool_size: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        let parsed: HashMap<String, toml::Value> = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        Self::from_map(&parsed)
    }
    
    fn from_map(map: &HashMap<String, toml::Value>) -> Result<Self, ConfigError> {
        let server = Self::parse_server(map)?;
        let database = Self::parse_database(map)?;
        let logging = Self::parse_logging(map)?;
        
        Ok(AppConfig {
            server,
            database,
            logging,
        })
    }
    
    fn parse_server(map: &HashMap<String, toml::Value>) -> Result<ServerConfig, ConfigError> {
        let server_table = map.get("server")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::MissingSection("server".to_string()))?;
        
        let host = server_table.get("host")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string());
        
        let port = server_table.get("port")
            .and_then(|v| v.as_integer())
            .map(|p| p as u16)
            .unwrap_or(8080);
        
        let enable_ssl = server_table.get("enable_ssl")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        Ok(ServerConfig {
            host,
            port,
            enable_ssl,
        })
    }
    
    fn parse_database(map: &HashMap<String, toml::Value>) -> Result<DatabaseConfig, ConfigError> {
        let db_table = map.get("database")
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::MissingSection("database".to_string()))?;
        
        let connection_string = db_table.get("connection_string")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ConfigError::MissingField("connection_string".to_string()))?;
        
        let pool_size = db_table.get("pool_size")
            .and_then(|v| v.as_integer())
            .map(|p| p as u32)
            .unwrap_or(10);
        
        let timeout_seconds = db_table.get("timeout_seconds")
            .and_then(|v| v.as_integer())
            .map(|t| t as u32)
            .unwrap_or(30);
        
        Ok(DatabaseConfig {
            connection_string,
            pool_size,
            timeout_seconds,
        })
    }
    
    fn parse_logging(map: &HashMap<String, toml::Value>) -> Result<LoggingConfig, ConfigError> {
        let logging_table = map.get("logging")
            .and_then(|v| v.as_table())
            .unwrap_or(&toml::value::Table::new());
        
        let level = logging_table.get("level")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "info".to_string());
        
        let file_path = logging_table.get("file_path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let enable_console = logging_table.get("enable_console")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        Ok(LoggingConfig {
            level,
            file_path,
            enable_console,
        })
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.server.port == 0 {
            errors.push("Server port cannot be zero".to_string());
        }
        
        if self.database.pool_size == 0 {
            errors.push("Database pool size cannot be zero".to_string());
        }
        
        let valid_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            errors.push(format!("Invalid log level: {}", self.logging.level));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    MissingSection(String),
    MissingField(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::MissingSection(section) => write!(f, "Missing section: {}", section),
            ConfigError::MissingField(field) => write!(f, "Missing field: {}", field),
        }
    }
}

impl std::error::Error for ConfigError {}