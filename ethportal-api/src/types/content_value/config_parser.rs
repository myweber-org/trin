use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: Vec<String>,
    pub timeout_seconds: u64,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                config_map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }

        let database_url = config_map
            .get("DATABASE_URL")
            .ok_or("Missing DATABASE_URL")?
            .to_string();

        let port = config_map
            .get("PORT")
            .map(|s| s.parse::<u16>())
            .unwrap_or(Ok(8080))
            .map_err(|e| format!("Invalid PORT: {}", e))?;

        let debug_mode = config_map
            .get("DEBUG")
            .map(|s| s.parse::<bool>())
            .unwrap_or(Ok(false))
            .map_err(|e| format!("Invalid DEBUG flag: {}", e))?;

        let api_keys = config_map
            .get("API_KEYS")
            .map(|s| s.split(',').map(|key| key.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);

        let timeout_seconds = config_map
            .get("TIMEOUT_SECONDS")
            .map(|s| s.parse::<u64>())
            .unwrap_or(Ok(30))
            .map_err(|e| format!("Invalid TIMEOUT_SECONDS: {}", e))?;

        Ok(Config {
            database_url,
            port,
            debug_mode,
            api_keys,
            timeout_seconds,
        })
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("DATABASE_URL cannot be empty".to_string());
        }

        if self.port == 0 {
            errors.push("PORT must be greater than 0".to_string());
        }

        if self.timeout_seconds == 0 {
            errors.push("TIMEOUT_SECONDS must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(temp_file, "PORT=3000").unwrap();
        writeln!(temp_file, "DEBUG=true").unwrap();
        writeln!(temp_file, "API_KEYS=key1,key2,key3").unwrap();
        writeln!(temp_file, "TIMEOUT_SECONDS=60").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.port, 3000);
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.api_keys, vec!["key1", "key2", "key3"]);
        assert_eq!(config.timeout_seconds, 60);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_with_defaults() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/test").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.port, 8080);
        assert_eq!(config.debug_mode, false);
        assert!(config.api_keys.is_empty());
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "PORT=not_a_number").unwrap();

        let result = Config::from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}