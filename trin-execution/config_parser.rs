
use std::collections::HashMap;
use std::env;
use std::fs;
use regex::Regex;

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut settings = HashMap::new();
        let var_regex = Regex::new(r"\$\{([A-Za-z0-9_]+)\}").unwrap();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some((key, mut value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                
                for cap in var_regex.captures_iter(&value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            value = value.replace(&cap[0], &env_value);
                        }
                    }
                }
                
                settings.insert(key, value.trim().to_string());
            }
        }
        
        Ok(Config { settings })
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut values = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
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
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_HOST=localhost").unwrap();
        writeln!(file, "DATABASE_PORT=5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "API_KEY=secret123").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(config.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(config.get("NON_EXISTENT"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secure_pass");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DB_PASS=${DB_PASSWORD}").unwrap();
        writeln!(file, "NO_SUBST=regular_value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DB_PASS"), Some(&"secure_pass".to_string()));
        assert_eq!(config.get("NO_SUBST"), Some(&"regular_value".to_string()));
    }

    #[test]
    fn test_missing_env_fallback() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "MISSING=${NON_EXISTENT_VAR}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("MISSING"), Some(&"${NON_EXISTENT_VAR}".to_string()));
    }
}