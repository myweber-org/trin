use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub max_connections: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/appdb"),
            log_level: String::from("info"),
            max_connections: 10,
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;
        
        config.validate()?;
        Ok(config)
    }

    pub fn from_file_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {}. Using defaults.", e);
                Self::default()
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err(String::from("Server port cannot be zero"));
        }
        
        if self.max_connections == 0 {
            return Err(String::from("Max connections must be greater than zero"));
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!(
                "Invalid log level: {}. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_valid_config_loading() {
        let toml_content = r#"
            server_port = 3000
            database_url = "postgresql://localhost:5432/testdb"
            log_level = "debug"
            max_connections = 20
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.max_connections, 20);
    }

    #[test]
    fn test_invalid_log_level() {
        let toml_content = r#"
            server_port = 3000
            database_url = "postgresql://localhost:5432/testdb"
            log_level = "invalid"
            max_connections = 20
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let result = AppConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_port_validation() {
        let toml_content = r#"
            server_port = 0
            database_url = "postgresql://localhost:5432/testdb"
            log_level = "info"
            max_connections = 20
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let result = AppConfig::from_file(temp_file.path());
        assert!(result.is_err());
    }
}