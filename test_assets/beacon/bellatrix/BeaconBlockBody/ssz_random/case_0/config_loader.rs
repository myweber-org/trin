use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config: HashMap<String, serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON format: {}", e))?;

        let database_url = Self::get_string(&mut config, "DATABASE_URL", "database_url")?;
        let server_port = Self::get_u16(&mut config, "SERVER_PORT", "server_port")?;
        let log_level = Self::get_string(&mut config, "LOG_LEVEL", "log_level")?;

        let features = config
            .remove("features")
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(k, v)| v.as_bool().map(|b| (k, b)))
            .collect();

        Ok(Config {
            database_url,
            server_port,
            log_level,
            features,
        })
    }

    fn get_string(
        config: &mut HashMap<String, serde_json::Value>,
        env_var: &str,
        key: &str,
    ) -> Result<String, String> {
        if let Ok(val) = env::var(env_var) {
            return Ok(val);
        }

        config
            .remove(key)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| format!("Missing required field: {}", key))
    }

    fn get_u16(
        config: &mut HashMap<String, serde_json::Value>,
        env_var: &str,
        key: &str,
    ) -> Result<u16, String> {
        if let Ok(val) = env::var(env_var) {
            return val
                .parse()
                .map_err(|e| format!("Invalid port number in {}: {}", env_var, e));
        }

        config
            .remove(key)
            .and_then(|v| v.as_u64().map(|n| n as u16))
            .ok_or_else(|| format!("Missing required field: {}", key))
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
        let config_content = r#"{
            "database_url": "postgres://localhost/test",
            "server_port": 8080,
            "log_level": "info",
            "features": {
                "caching": true,
                "metrics": false
            }
        }"#;

        write!(temp_file, "{}", config_content).unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.features.get("caching"), Some(&true));
        assert_eq!(config.features.get("metrics"), Some(&false));
    }

    #[test]
    fn test_env_var_override() {
        env::set_var("DATABASE_URL", "postgres://prod/db");
        env::set_var("SERVER_PORT", "3000");

        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"{
            "database_url": "postgres://localhost/test",
            "server_port": 8080,
            "log_level": "debug"
        }"#;

        write!(temp_file, "{}", config_content).unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://prod/db");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "debug");

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }
}