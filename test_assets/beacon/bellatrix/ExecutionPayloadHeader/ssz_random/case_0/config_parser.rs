use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                config_map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = Self::get_value(map, "DATABASE_URL")?;
        let port_str = Self::get_value(map, "PORT")?;
        let port = port_str
            .parse()
            .map_err(|_| format!("Invalid port number: {}", port_str))?;
        let debug_mode = Self::get_value(map, "DEBUG")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        let mut api_keys = HashMap::new();
        for (key, value) in map {
            if key.starts_with("API_KEY_") {
                api_keys.insert(key.clone(), value.clone());
            }
        }

        Ok(Config {
            database_url,
            port,
            debug_mode,
            api_keys,
        })
    }

    fn get_value(map: &HashMap<String, String>, key: &str) -> Result<String, String> {
        map.get(key)
            .map(|s| s.clone())
            .or_else(|| env::var(key).ok())
            .ok_or_else(|| format!("Missing required configuration: {}", key))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        if self.port == 0 {
            return Err("Port must be greater than 0".to_string());
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
        writeln!(temp_file, "PORT=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "DEBUG=true").unwrap();
        writeln!(temp_file, "API_KEY_WEATHER=abc123").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.port, 8080);
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.api_keys.get("API_KEY_WEATHER"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_env_fallback() {
        env::set_var("DATABASE_URL", "postgres://env/test");
        let map = HashMap::from([
            ("PORT".to_string(), "3000".to_string()),
        ]);
        
        let config = Config::from_map(&map).unwrap();
        assert_eq!(config.database_url, "postgres://env/test");
        env::remove_var("DATABASE_URL");
    }
}