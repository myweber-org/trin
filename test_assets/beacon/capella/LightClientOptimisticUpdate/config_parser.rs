
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
    pub fn new() -> Result<Self, String> {
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
        
        let config_table: toml::Value = config_content
            .parse()
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        
        let database_url = get_config_value(&config_table, "database.url")
            .or_else(|| env::var("DATABASE_URL").ok())
            .ok_or("Database URL not found in config or environment")?;
        
        let port = get_config_value(&config_table, "server.port")
            .and_then(|s| s.parse::<u16>().ok())
            .or_else(|| env::var("PORT").ok().and_then(|s| s.parse::<u16>().ok()))
            .unwrap_or(8080);
        
        let debug_mode = get_config_value(&config_table, "debug.enabled")
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false);
        
        let mut api_keys = HashMap::new();
        if let Some(keys_table) = config_table.get("api_keys") {
            if let Some(table) = keys_table.as_table() {
                for (key, value) in table {
                    if let Some(val_str) = value.as_str() {
                        api_keys.insert(key.clone(), val_str.to_string());
                    }
                }
            }
        }
        
        Ok(Config {
            database_url,
            port,
            debug_mode,
            api_keys,
        })
    }
    
    pub fn get_api_key(&self, service: &str) -> Option<&String> {
        self.api_keys.get(service)
    }
}

fn get_config_value(config: &toml::Value, path: &str) -> Option<String> {
    let mut current = config;
    
    for part in path.split('.') {
        current = current.get(part)?;
    }
    
    current.as_str().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            [database]
            url = "postgresql://localhost/mydb"
            
            [server]
            port = 3000
            
            [debug]
            enabled = true
            
            [api_keys]
            stripe = "sk_test_123"
            google = "google_api_key_456"
        "#;
        
        writeln!(temp_file, "{}", config_content).unwrap();
        
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        
        let config = Config::new().unwrap();
        
        assert_eq!(config.database_url, "postgresql://localhost/mydb");
        assert_eq!(config.port, 3000);
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.get_api_key("stripe"), Some(&"sk_test_123".to_string()));
        assert_eq!(config.get_api_key("google"), Some(&"google_api_key_456".to_string()));
    }
    
    #[test]
    fn test_environment_override() {
        env::set_var("DATABASE_URL", "postgresql://prod-server/proddb");
        env::set_var("PORT", "9000");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
            [database]
            url = "postgresql://localhost/mydb"
            
            [server]
            port = 3000
        "#;
        
        writeln!(temp_file, "{}", config_content).unwrap();
        env::set_var("CONFIG_PATH", temp_file.path().to_str().unwrap());
        
        let config = Config::new().unwrap();
        
        assert_eq!(config.database_url, "postgresql://prod-server/proddb");
        assert_eq!(config.port, 9000);
        
        env::remove_var("DATABASE_URL");
        env::remove_var("PORT");
    }
}