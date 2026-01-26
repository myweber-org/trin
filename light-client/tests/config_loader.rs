use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub pool_size: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub enable_https: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_files: usize,
    pub max_file_size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub environment: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "".to_string(),
            database_name: "app_db".to_string(),
            pool_size: 10,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            address: "0.0.0.0".to_string(),
            port: 8080,
            enable_https: false,
            cert_path: None,
            key_path: None,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: "info".to_string(),
            file_path: None,
            max_files: 5,
            max_file_size: 10_485_760,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            environment: "development".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config file: {}. Using defaults.", e);
                AppConfig::default()
            }
        }
    }

    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.database.port == 0 {
            return Err("Database port cannot be zero".into());
        }

        if self.server.port == 0 {
            return Err("Server port cannot be zero".into());
        }

        if self.server.enable_https {
            if self.server.cert_path.is_none() || self.server.key_path.is_none() {
                return Err("HTTPS requires both certificate and key paths".into());
            }
        }

        let valid_envs = ["development", "staging", "production"];
        if !valid_envs.contains(&self.environment.as_str()) {
            return Err(format!(
                "Invalid environment: {}. Must be one of: {:?}",
                self.environment, valid_envs
            )
            .into());
        }

        Ok(())
    }

    pub fn to_toml_string(&self) -> Result<String, Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        Ok(toml_string)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = self.to_toml_string()?;
        fs::write(path, toml_string)?;
        Ok(())
    }
}