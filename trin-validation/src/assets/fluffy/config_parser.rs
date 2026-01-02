
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut values = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let processed_value = Self::process_value(value.trim());
                values.insert(key.trim().to_string(), processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len() - 1];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values
            .get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=${DB_PASSWORD}").unwrap();
        writeln!(file, "HOST=${UNDEFINED_VAR}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("HOST"), Some(&"${UNDEFINED_VAR}".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
    pub debug_mode: bool,
    pub api_keys: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut settings = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let raw_value = parts[1].trim().to_string();
                let value = Self::resolve_env_vars(&raw_value);
                settings.insert(key, value);
            }
        }
        
        let database_url = settings
            .get("DATABASE_URL")
            .ok_or("Missing DATABASE_URL setting")?
            .clone();
        
        let max_connections = settings
            .get("MAX_CONNECTIONS")
            .unwrap_or(&"10".to_string())
            .parse()
            .map_err(|_| "Invalid MAX_CONNECTIONS value")?;
        
        let debug_mode = settings
            .get("DEBUG_MODE")
            .unwrap_or(&"false".to_string())
            .parse()
            .unwrap_or(false);
        
        let api_keys = settings
            .get("API_KEYS")
            .map(|s| s.split(',').map(|k| k.trim().to_string()).collect())
            .unwrap_or_else(Vec::new);
        
        Ok(Config {
            database_url,
            max_connections,
            debug_mode,
            api_keys,
        })
    }
    
    fn resolve_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        if let Some(start) = result.find("${") {
            if let Some(end) = result[start..].find('}') {
                let full_match = &result[start..=start + end];
                let var_name = &full_match[2..full_match.len() - 1];
                if let Ok(env_value) = env::var(var_name) {
                    result = result.replace(full_match, &env_value);
                }
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
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(config_file, "MAX_CONNECTIONS=25").unwrap();
        writeln!(config_file, "DEBUG_MODE=true").unwrap();
        writeln!(config_file, "API_KEYS=key1,key2,key3").unwrap();
        
        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.database_url, "postgres://localhost/db");
        assert_eq!(config.max_connections, 25);
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.api_keys, vec!["key1", "key2", "key3"]);
    }
    
    #[test]
    fn test_env_var_substitution() {
        env::set_var("DB_HOST", "localhost");
        
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(config_file, "DATABASE_URL=postgres://${DB_HOST}/production").unwrap();
        
        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/production");
    }
}