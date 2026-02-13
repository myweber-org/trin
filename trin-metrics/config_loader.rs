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
        let mut config: HashMap<String, String> = serde_json::from_str(&contents)?;

        Self::apply_env_overrides(&mut config);

        Ok(Config {
            database_url: Self::get_value(&config, "DATABASE_URL")?,
            port: Self::get_value(&config, "PORT")?.parse()?,
            debug_mode: Self::get_value(&config, "DEBUG")?.parse()?,
            api_keys: Self::parse_api_keys(&config),
        })
    }

    fn apply_env_overrides(config: &mut HashMap<String, String>) {
        for (key, value) in env::vars() {
            if config.contains_key(&key) {
                config.insert(key, value);
            }
        }
    }

    fn get_value(config: &HashMap<String, String>, key: &str) -> Result<String, String> {
        config
            .get(key)
            .cloned()
            .ok_or_else(|| format!("Missing configuration key: {}", key))
    }

    fn parse_api_keys(config: &HashMap<String, String>) -> HashMap<String, String> {
        config
            .iter()
            .filter(|(k, _)| k.starts_with("API_KEY_"))
            .map(|(k, v)| (k.replace("API_KEY_", ""), v.clone()))
            .collect()
    }
}