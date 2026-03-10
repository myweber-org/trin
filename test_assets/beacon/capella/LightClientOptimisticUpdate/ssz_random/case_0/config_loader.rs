use std::collections::HashMap;
use std::env;
use std::fs;
use serde_yaml;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
    pub allowed_hosts: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.yaml".to_string());

        let file_contents = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;

        let mut config: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(&file_contents)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;

        let database_url = Self::get_string(&mut config, "database_url")
            .or_else(|_| env::var("DATABASE_URL"))
            .map_err(|_| "Database URL not found in config or environment".to_string())?;

        let api_key = Self::get_string(&mut config, "api_key")
            .or_else(|_| env::var("API_KEY"))
            .map_err(|_| "API key not found in config or environment".to_string())?;

        let debug_mode = Self::get_bool(&mut config, "debug_mode")
            .unwrap_or_else(|_| env::var("DEBUG_MODE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false));

        let port = Self::get_u16(&mut config, "port")
            .or_else(|_| env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok()))
            .unwrap_or(8080);

        let allowed_hosts = Self::get_string_array(&mut config, "allowed_hosts")
            .unwrap_or_else(|_| vec!["localhost".to_string(), "127.0.0.1".to_string()]);

        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
            allowed_hosts,
        })
    }

    fn get_string(config: &mut HashMap<String, serde_yaml::Value>, key: &str) -> Result<String, ()> {
        config.remove(key)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or(())
    }

    fn get_bool(config: &mut HashMap<String, serde_yaml::Value>, key: &str) -> Result<bool, ()> {
        config.remove(key)
            .and_then(|v| v.as_bool())
            .ok_or(())
    }

    fn get_u16(config: &mut HashMap<String, serde_yaml::Value>, key: &str) -> Result<u16, ()> {
        config.remove(key)
            .and_then(|v| v.as_u64())
            .and_then(|v| if v <= u16::MAX as u64 { Some(v as u16) } else { None })
            .ok_or(())
    }

    fn get_string_array(config: &mut HashMap<String, serde_yaml::Value>, key: &str) -> Result<Vec<String>, ()> {
        config.remove(key)
            .and_then(|v| v.as_sequence())
            .map(|seq| seq.iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect())
            .ok_or(())
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
        let config_content = r#"
database_url: "postgres://localhost/test"
api_key: "test_key_123"
debug_mode: true
port: 3000
allowed_hosts:
  - "example.com"
  - "api.example.com"
"#;
        write!(temp_file, "{}", config_content).unwrap();

        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());

        let config = Config::load().unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.api_key, "test_key_123");
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.port, 3000);
        assert_eq!(config.allowed_hosts, vec!["example.com", "api.example.com"]);

        env::remove_var("CONFIG_PATH");
    }

    #[test]
    fn test_environment_override() {
        env::set_var("DATABASE_URL", "postgres://prod/db");
        env::set_var("API_KEY", "prod_key_456");
        env::set_var("DEBUG_MODE", "false");
        env::set_var("PORT", "9000");

        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
database_url: "should_be_overridden"
api_key: "should_be_overridden"
debug_mode: true
port: 3000
"#;
        write!(temp_file, "{}", config_content).unwrap();
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());

        let config = Config::load().unwrap();
        assert_eq!(config.database_url, "postgres://prod/db");
        assert_eq!(config.api_key, "prod_key_456");
        assert_eq!(config.debug_mode, false);
        assert_eq!(config.port, 9000);

        env::remove_var("CONFIG_PATH");
        env::remove_var("DATABASE_URL");
        env::remove_var("API_KEY");
        env::remove_var("DEBUG_MODE");
        env::remove_var("PORT");
    }
}