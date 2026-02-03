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
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
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

    fn from_map(map: &HashMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = Self::get_value(map, "DATABASE_URL")?;
        let port_str = Self::get_value(map, "PORT")?;
        let port = port_str.parse::<u16>()?;
        let debug_str = Self::get_value(map, "DEBUG").unwrap_or_else(|_| "false".to_string());
        let debug_mode = debug_str.to_lowercase() == "true";

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

    fn get_value(map: &HashMap<String, String>, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(value) = env::var(key).ok().or_else(|| map.get(key).cloned()) {
            Ok(value)
        } else {
            Err(format!("Missing required configuration: {}", key).into())
        }
    }

    pub fn get_api_key(&self, service: &str) -> Option<&String> {
        let key_name = format!("API_KEY_{}", service.to_uppercase());
        self.api_keys.get(&key_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(temp_file, "PORT=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "DEBUG=true").unwrap();
        writeln!(temp_file, "API_KEY_WEATHER=abc123").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.port, 8080);
        assert!(config.debug_mode);
        assert_eq!(config.get_api_key("weather"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_env_var_override() {
        env::set_var("DATABASE_URL", "postgres://prod/db");
        
        let mut map = HashMap::new();
        map.insert("PORT".to_string(), "3000".to_string());
        
        let config = Config::from_map(&map).unwrap();
        assert_eq!(config.database_url, "postgres://prod/db");
        assert_eq!(config.port, 3000);
        assert!(!config.debug_mode);
        
        env::remove_var("DATABASE_URL");
    }
}