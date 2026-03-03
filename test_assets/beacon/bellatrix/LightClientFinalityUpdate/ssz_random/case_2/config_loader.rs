
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub feature_flags: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: HashMap<String, String> = serde_json::from_str(&content)?;
        
        Self::apply_env_overrides(&mut config);
        
        Ok(Config {
            database_url: Self::get_value(&config, "DATABASE_URL", "database_url"),
            server_port: Self::get_value(&config, "SERVER_PORT", "server_port")
                .parse()
                .unwrap_or(8080),
            log_level: Self::get_value(&config, "LOG_LEVEL", "log_level"),
            feature_flags: Self::parse_feature_flags(&config),
        })
    }
    
    fn apply_env_overrides(config: &mut HashMap<String, String>) {
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                let config_key = key.trim_start_matches("APP_").to_lowercase();
                config.insert(config_key, value);
            }
        }
    }
    
    fn get_value(config: &HashMap<String, String>, env_var: &str, config_key: &str) -> String {
        env::var(env_var)
            .ok()
            .or_else(|| config.get(config_key).cloned())
            .unwrap_or_else(|| match config_key {
                "database_url" => "postgres://localhost:5432/app".to_string(),
                "log_level" => "info".to_string(),
                _ => String::new(),
            })
    }
    
    fn parse_feature_flags(config: &HashMap<String, String>) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        
        for (key, value) in config {
            if key.starts_with("feature_") {
                let flag_name = key.trim_start_matches("feature_").to_string();
                let enabled = value.to_lowercase() == "true" || value == "1";
                flags.insert(flag_name, enabled);
            }
        }
        
        flags
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.feature_flags.get(feature).copied().unwrap_or(false)
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
        let config_data = r#"{
            "database_url": "postgres://test:5432/db",
            "server_port": "3000",
            "log_level": "debug",
            "feature_new_ui": "true"
        }"#;
        
        write!(temp_file, "{}", config_data).unwrap();
        
        env::set_var("APP_LOG_LEVEL", "trace");
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.database_url, "postgres://test:5432/db");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "trace");
        assert!(config.is_feature_enabled("new_ui"));
        
        env::remove_var("APP_LOG_LEVEL");
    }
}