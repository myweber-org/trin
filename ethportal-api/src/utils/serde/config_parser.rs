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
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
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
        writeln!(file, "PASSWORD=$DB_PASSWORD").unwrap();
        writeln!(file, "NORMAL=value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NORMAL"), Some(&"value".to_string()));
    }
}use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub cache_size: usize,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config_map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", trimmed));
            }
            
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            config_map.insert(key, value);
        }
        
        Self::from_map(config_map)
    }
    
    fn from_map(map: HashMap<String, String>) -> Result<Self, String> {
        let database_url = map.get("database_url")
            .ok_or("Missing required field: database_url")?
            .clone();
        
        let port = map.get("port")
            .map(|s| s.parse::<u16>())
            .unwrap_or(Ok(8080))
            .map_err(|e| format!("Invalid port value: {}", e))?;
        
        let log_level = map.get("log_level")
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "info".to_string());
        
        let cache_size = map.get("cache_size")
            .map(|s| s.parse::<usize>())
            .unwrap_or(Ok(1000))
            .map_err(|e| format!("Invalid cache_size value: {}", e))?;
        
        let mut features = HashMap::new();
        for (key, value) in map {
            if key.starts_with("feature.") {
                let feature_name = key.trim_start_matches("feature.").to_string();
                let enabled = value.parse::<bool>()
                    .map_err(|e| format!("Invalid boolean value for feature {}: {}", feature_name, e))?;
                features.insert(feature_name, enabled);
            }
        }
        
        Ok(Config {
            database_url,
            port,
            log_level,
            cache_size,
            features,
        })
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }
        
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        Ok(())
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "database_url=postgres://localhost/mydb").unwrap();
        writeln!(file, "port=5432").unwrap();
        writeln!(file, "log_level=debug").unwrap();
        writeln!(file, "cache_size=5000").unwrap();
        writeln!(file, "feature.caching=true").unwrap();
        writeln!(file, "feature.analytics=false").unwrap();
        
        let config = Config::from_file(file.path()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/mydb");
        assert_eq!(config.port, 5432);
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_size, 5000);
        assert!(config.is_feature_enabled("caching"));
        assert!(!config.is_feature_enabled("analytics"));
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_with_defaults() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "database_url=postgres://localhost/test").unwrap();
        
        let config = Config::from_file(file.path()).unwrap();
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_size, 1000);
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "port=not_a_number").unwrap();
        
        let result = Config::from_file(file.path());
        assert!(result.is_err());
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut settings = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::interpolate_env_vars(value.trim());
                settings.insert(key, processed_value);
            }
        }

        Ok(Config { settings })
    }

    fn interpolate_env_vars(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        let mut in_braces = false;
        let mut var_name = String::new();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next();
                in_braces = true;
                var_name.clear();
                continue;
            }

            if in_braces {
                if ch == '}' {
                    let env_value = env::var(&var_name).unwrap_or_default();
                    result.push_str(&env_value);
                    in_braces = false;
                } else {
                    var_name.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.get(key).cloned().unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let content = "HOST=localhost\nPORT=8080\nDEBUG=true\n";
        let config = Config::from_str(content).unwrap();
        
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_var_interpolation() {
        env::set_var("APP_SECRET", "super-secret-key");
        
        let content = "SECRET=${APP_SECRET}\nHOST=localhost";
        let config = Config::from_str(content).unwrap();
        
        assert_eq!(config.get("SECRET"), Some(&"super-secret-key".to_string()));
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
    }

    #[test]
    fn test_file_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        
        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some(&"postgres://localhost/db".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let content = "EXISTING=value";
        let config = Config::from_str(content).unwrap();
        
        assert_eq!(config.get_or_default("EXISTING", "default"), "value");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        self.parse_content(&content)
    }

    fn parse_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = self.process_value(value.trim());
                self.values.insert(key, processed_value);
            }
        }
        Ok(())
    }

    fn process_value(&self, value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let env_var = &value[2..value.len() - 1];
            env::var(env_var).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or(default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut config = ConfigParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_HOST=localhost").unwrap();
        writeln!(temp_file, "DATABASE_PORT=5432").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();

        config.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(config.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "env_secret_value");
        
        let mut config = ConfigParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "SECRET=${APP_SECRET}").unwrap();
        writeln!(temp_file, "NORMAL=regular_value").unwrap();

        config.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.get("SECRET"), Some(&"env_secret_value".to_string()));
        assert_eq!(config.get("NORMAL"), Some(&"regular_value".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let config = ConfigParser::new();
        assert_eq!(config.get_or_default("MISSING", "default_value"), "default_value");
    }
}