
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: Vec<String>,
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
                config_map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }

        Self::from_map(&config_map)
    }

    pub fn from_env() -> Result<Self, String> {
        let mut config_map = HashMap::new();
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                config_map.insert(key.trim_start_matches("APP_").to_string(), value);
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = map
            .get("DATABASE_URL")
            .map(|s| s.to_string())
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://localhost:5432/app".to_string());

        let port = map
            .get("PORT")
            .and_then(|s| s.parse().ok())
            .or_else(|| env::var("PORT").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(8080);

        let debug_mode = map
            .get("DEBUG")
            .map(|s| s.to_lowercase() == "true")
            .or_else(|| env::var("DEBUG").ok().map(|s| s.to_lowercase() == "true"))
            .unwrap_or(false);

        let api_keys = map
            .get("API_KEYS")
            .map(|s| s.split(',').map(|key| key.trim().to_string()).collect())
            .or_else(|| {
                env::var("API_KEYS")
                    .ok()
                    .map(|s| s.split(',').map(|key| key.trim().to_string()).collect())
            })
            .unwrap_or_else(Vec::new);

        Ok(Config {
            database_url,
            port,
            debug_mode,
            api_keys,
        })
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            database_url: other.database_url,
            port: other.port,
            debug_mode: other.debug_mode,
            api_keys: if other.api_keys.is_empty() {
                self.api_keys
            } else {
                other.api_keys
            },
        }
    }
}

pub fn load_config() -> Result<Config, String> {
    let file_config = Config::from_file("config/app.conf").unwrap_or_default();
    let env_config = Config::from_env()?;
    Ok(file_config.merge(env_config))
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgres://localhost:5432/app".to_string(),
            port: 8080,
            debug_mode: false,
            api_keys: Vec::new(),
        }
    }
}