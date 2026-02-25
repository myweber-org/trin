use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config = Config::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                config.values.insert(key, value);
            }
        }

        Ok(config)
    }

    pub fn get(&self, key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .or_else(|| self.values.get(key).cloned())
    }

    pub fn get_with_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key) || env::var(key).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_creation() {
        let config = Config::new();
        assert!(config.values.is_empty());
    }

    #[test]
    fn test_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost:5432").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "API_KEY=secret123").unwrap();

        let config = Config::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(config.get("API_KEY").unwrap(), "secret123");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_environment_override() {
        env::set_var("TEST_KEY", "env_value");
        let mut config = Config::new();
        config.set("TEST_KEY", "file_value");

        assert_eq!(config.get("TEST_KEY").unwrap(), "env_value");
        env::remove_var("TEST_KEY");
    }

    #[test]
    fn test_get_with_default() {
        let config = Config::new();
        assert_eq!(config.get_with_default("MISSING", "default_value"), "default_value");
    }
}
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
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(raw: &str) -> String {
        let mut result = String::new();
        let mut chars = raw.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '}' {
                        chars.next(); // Skip '}'
                        break;
                    }
                    var_name.push(ch);
                    chars.next();
                }
                
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
                } else {
                    result.push_str(&format!("${{{}}}", var_name));
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
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
        writeln!(file, "  TIMEOUT = 30  ").unwrap();

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
        writeln!(file, "PASSWORD=${{DB_PASSWORD}}").unwrap();
        writeln!(file, "URL=postgres://user:${{DB_PASSWORD}}@localhost").unwrap();
        writeln!(file, "MISSING=${{UNDEFINED_VAR}}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("URL"), Some(&"postgres://user:secret123@localhost".to_string()));
        assert_eq!(config.get("MISSING"), Some(&"${UNDEFINED_VAR}".to_string()));
    }
}