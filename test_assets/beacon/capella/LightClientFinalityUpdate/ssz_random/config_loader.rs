use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let mut config_map = HashMap::new();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = Self::resolve_env_vars(value.trim());
                config_map.insert(key, value);
            }
        }

        Self::from_map(&config_map)
    }

    fn from_map(map: &HashMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = map
            .get("DATABASE_URL")
            .ok_or("Missing DATABASE_URL")?
            .to_string();

        let port_str = map.get("PORT").ok_or("Missing PORT")?;
        let port = port_str.parse::<u16>()?;

        let debug_mode = map
            .get("DEBUG_MODE")
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

    fn resolve_env_vars(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("DATABASE_URL cannot be empty".to_string());
        }

        if self.port == 0 {
            errors.push("PORT must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}