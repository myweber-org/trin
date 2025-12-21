use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub debug_mode: bool,
    pub port: u16,
    pub custom_settings: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH")
            .unwrap_or_else(|_| "config.toml".to_string());

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;

        let parsed: toml::Value = config_content.parse()
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        let table = parsed.as_table()
            .ok_or_else(|| "Config file must be a TOML table".to_string())?;

        let database_url = Self::get_string(table, "database_url")?;
        let api_key = Self::get_string(table, "api_key")?;
        let debug_mode = Self::get_bool(table, "debug_mode")?;
        let port = Self::get_u16(table, "port")?;

        let custom_settings = Self::extract_custom_settings(table);

        Ok(Config {
            database_url,
            api_key,
            debug_mode,
            port,
            custom_settings,
        })
    }

    fn get_string(table: &toml::value::Table, key: &str) -> Result<String, String> {
        table.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Missing or invalid string field: {}", key))
    }

    fn get_bool(table: &toml::value::Table, key: &str) -> Result<bool, String> {
        table.get(key)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| format!("Missing or invalid boolean field: {}", key))
    }

    fn get_u16(table: &toml::value::Table, key: &str) -> Result<u16, String> {
        table.get(key)
            .and_then(|v| v.as_integer())
            .and_then(|i| i.try_into().ok())
            .ok_or_else(|| format!("Missing or invalid u16 field: {}", key))
    }

    fn extract_custom_settings(table: &toml::value::Table) -> HashMap<String, String> {
        let mut settings = HashMap::new();
        let reserved_keys = ["database_url", "api_key", "debug_mode", "port"];

        for (key, value) in table {
            if reserved_keys.contains(&key.as_str()) {
                continue;
            }

            if let Some(str_val) = value.as_str() {
                settings.insert(key.clone(), str_val.to_string());
            }
        }

        settings
    }
}