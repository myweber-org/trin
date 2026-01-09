use std::collections::HashMap;
use std::fs;

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
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|_| ConfigError::FileNotFound(path.to_string()))?;

        let parsed: toml::Value = content
            .parse()
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Self::validate_and_build(parsed)
    }

    fn validate_and_build(value: toml::Value) -> Result<Self, ConfigError> {
        let table = value.as_table()
            .ok_or_else(|| ConfigError::ValidationError("Root must be a table".to_string()))?;

        let database = Self::parse_database(table.get("database"))?;
        let server = Self::parse_server(table.get("server"))?;
        let features = Self::parse_features(table.get("features"));

        Ok(Config {
            database,
            server,
            features,
        })
    }

    fn parse_database(value: Option<&toml::Value>) -> Result<DatabaseConfig, ConfigError> {
        let db_table = value
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::ValidationError("Missing or invalid database section".to_string()))?;

        let host = db_table.get("host")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ConfigError::ValidationError("Database host is required".to_string()))?;

        let port = db_table.get("port")
            .and_then(|v| v.as_integer())
            .map(|p| p as u16)
            .ok_or_else(|| ConfigError::ValidationError("Database port is required".to_string()))?;

        let username = db_table.get("username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ConfigError::ValidationError("Database username is required".to_string()))?;

        let password = db_table.get("password")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ConfigError::ValidationError("Database password is required".to_string()))?;

        Ok(DatabaseConfig {
            host,
            port,
            username,
            password,
        })
    }

    fn parse_server(value: Option<&toml::Value>) -> Result<ServerConfig, ConfigError> {
        let server_table = value
            .and_then(|v| v.as_table())
            .ok_or_else(|| ConfigError::ValidationError("Missing or invalid server section".to_string()))?;

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

    fn parse_features(value: Option<&toml::Value>) -> HashMap<String, bool> {
        let mut features = HashMap::new();
        
        if let Some(features_table) = value.and_then(|v| v.as_table()) {
            for (key, val) in features_table {
                if let Some(bool_val) = val.as_bool() {
                    features.insert(key.clone(), bool_val);
                }
            }
        }
        
        features
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let toml_content = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "admin"
            password = "secret"

            [server]
            host = "0.0.0.0"
            port = 8080
            enable_ssl = true

            [features]
            logging = true
            caching = false
        "#;

        let parsed: toml::Value = toml_content.parse().unwrap();
        let config = Config::validate_and_build(parsed).unwrap();

        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert!(config.server.enable_ssl);
        assert_eq!(config.features.get("logging"), Some(&true));
        assert_eq!(config.features.get("caching"), Some(&false));
    }

    #[test]
    fn test_missing_database() {
        let toml_content = r#"
            [server]
            host = "0.0.0.0"
        "#;

        let parsed: toml::Value = toml_content.parse().unwrap();
        let result = Config::validate_and_build(parsed);
        
        assert!(matches!(result, Err(ConfigError::ValidationError(_))));
    }
}