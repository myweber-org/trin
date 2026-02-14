
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                config_map.insert(key, value);
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = Self::get_value(map, "DATABASE_URL")?;
        let server_port = Self::get_value(map, "SERVER_PORT")?
            .parse::<u16>()
            .map_err(|e| format!("Invalid port number: {}", e))?;
        let log_level = Self::get_value(map, "LOG_LEVEL")?;
        let cache_ttl = Self::get_value(map, "CACHE_TTL")?
            .parse::<u64>()
            .map_err(|e| format!("Invalid cache TTL: {}", e))?;

        Ok(Config {
            database_url,
            server_port,
            log_level,
            cache_ttl,
        })
    }

    fn get_value(map: &HashMap<String, String>, key: &str) -> Result<String, String> {
        map.get(key)
            .map(|s| s.to_string())
            .or_else(|| env::var(key).ok())
            .ok_or_else(|| format!("Missing required configuration: {}", key))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }

        if !self.database_url.starts_with("postgres://") {
            return Err("Database URL must use PostgreSQL protocol".to_string());
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(temp_file, "SERVER_PORT=8080").unwrap();
        writeln!(temp_file, "LOG_LEVEL=info").unwrap();
        writeln!(temp_file, "CACHE_TTL=300").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            database_url: "postgres://localhost/test".to_string(),
            server_port: 8080,
            log_level: "info".to_string(),
            cache_ttl: 300,
        };

        assert!(config.validate().is_ok());
    }
}