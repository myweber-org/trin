use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub features: HashMap<String, bool>,
}

#[derive(Debug, PartialEq)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound,
    ParseError(String),
    ValidationError(String),
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|_| ConfigError::FileNotFound)?;

        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        let mut database_host = String::new();
        let mut database_port = 0;
        let mut database_username = String::new();
        let mut database_password = String::new();
        let mut server_host = String::new();
        let mut server_port = 0;
        let mut enable_ssl = false;
        let mut features = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(ConfigError::ParseError(
                    format!("Invalid line format: {}", line)
                ));
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "database.host" => database_host = value.to_string(),
                "database.port" => {
                    database_port = value.parse()
                        .map_err(|_| ConfigError::ParseError(
                            format!("Invalid port number: {}", value)
                        ))?;
                }
                "database.username" => database_username = value.to_string(),
                "database.password" => database_password = value.to_string(),
                "server.host" => server_host = value.to_string(),
                "server.port" => {
                    server_port = value.parse()
                        .map_err(|_| ConfigError::ParseError(
                            format!("Invalid port number: {}", value)
                        ))?;
                }
                "server.enable_ssl" => {
                    enable_ssl = value.parse()
                        .map_err(|_| ConfigError::ParseError(
                            format!("Invalid boolean value: {}", value)
                        ))?;
                }
                _ if key.starts_with("feature.") => {
                    let feature_name = key.trim_start_matches("feature.").to_string();
                    let enabled = value.parse()
                        .map_err(|_| ConfigError::ParseError(
                            format!("Invalid boolean value for feature: {}", value)
                        ))?;
                    features.insert(feature_name, enabled);
                }
                _ => return Err(ConfigError::ParseError(
                    format!("Unknown configuration key: {}", key)
                )),
            }
        }

        let config = Config {
            database: DatabaseConfig {
                host: database_host,
                port: database_port,
                username: database_username,
                password: database_password,
            },
            server: ServerConfig {
                host: server_host,
                port: server_port,
                enable_ssl,
            },
            features,
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.database.host.is_empty() {
            return Err(ConfigError::ValidationError(
                "Database host cannot be empty".to_string()
            ));
        }

        if self.database.port == 0 {
            return Err(ConfigError::ValidationError(
                "Database port cannot be zero".to_string()
            ));
        }

        if self.server.port == 0 {
            return Err(ConfigError::ValidationError(
                "Server port cannot be zero".to_string()
            ));
        }

        if self.database.username.is_empty() {
            return Err(ConfigError::ValidationError(
                "Database username cannot be empty".to_string()
            ));
        }

        Ok(())
    }

    pub fn is_feature_enabled(&self, feature_name: &str) -> bool {
        self.features.get(feature_name).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let config_str = r#"
            database.host = localhost
            database.port = 5432
            database.username = admin
            database.password = secret
            server.host = 0.0.0.0
            server.port = 8080
            server.enable_ssl = true
            feature.cache = true
            feature.logging = false
        "#;

        let config = Config::from_str(config_str).unwrap();
        
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.database.username, "admin");
        assert_eq!(config.database.password, "secret");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert!(config.server.enable_ssl);
        assert!(config.is_feature_enabled("cache"));
        assert!(!config.is_feature_enabled("logging"));
    }

    #[test]
    fn test_invalid_port() {
        let config_str = r#"
            database.host = localhost
            database.port = not_a_number
            database.username = admin
            database.password = secret
            server.host = 0.0.0.0
            server.port = 8080
            server.enable_ssl = true
        "#;

        let result = Config::from_str(config_str);
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
    }

    #[test]
    fn test_missing_required_field() {
        let config_str = r#"
            database.port = 5432
            database.username = admin
            database.password = secret
            server.host = 0.0.0.0
            server.port = 8080
            server.enable_ssl = true
        "#;

        let result = Config::from_str(config_str);
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));
    }
}