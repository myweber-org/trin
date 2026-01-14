
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub max_connections: u32,
    pub debug_mode: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let parsed = Self::parse_config(&content)?;
        
        Ok(Config {
            database_url: parsed.get("database_url").unwrap().to_string(),
            api_key: parsed.get("api_key").unwrap().to_string(),
            max_connections: parsed.get("max_connections").unwrap().parse()?,
            debug_mode: parsed.get("debug_mode").unwrap().parse()?,
        })
    }

    fn parse_config(content: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut config_map = HashMap::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let mut value = parts[1].trim().to_string();
                
                value = Self::substitute_env_vars(&value);
                config_map.insert(key, value);
            }
        }
        
        Ok(config_map)
    }

    fn substitute_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
            if let Ok(env_value) = env::var(var_name) {
                result = env_value;
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "database_url=postgres://localhost/db").unwrap();
        writeln!(file, "api_key=${API_KEY}").unwrap();
        writeln!(file, "max_connections=10").unwrap();
        writeln!(file, "debug_mode=true").unwrap();
        
        env::set_var("API_KEY", "secret123");
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.api_key, "secret123");
        assert_eq!(config.max_connections, 10);
        assert!(config.debug_mode);
    }
}