
use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut values = HashMap::new();
        let var_regex = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                value = value.trim();

                let mut processed_value = value.to_string();
                for cap in var_regex.captures_iter(value) {
                    if let Some(var_name) = cap.get(1) {
                        let var_name = var_name.as_str();
                        if let Ok(env_value) = env::var(var_name) {
                            processed_value = processed_value.replace(&cap[0], &env_value);
                        }
                    }
                }

                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let content = "HOST=localhost\nPORT=8080\nDEBUG=true\n";
        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "postgresql");
        let content = "DATABASE_URL=${DB_HOST}://localhost/db";
        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("DATABASE_URL"), Some(&"postgresql://localhost/db".to_string()));
    }

    #[test]
    fn test_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "APP_NAME=test_app\nVERSION=1.0.0").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("APP_NAME"), Some(&"test_app".to_string()));
        assert_eq!(config.get("VERSION"), Some(&"1.0.0".to_string()));
    }
}use std::collections::HashMap;
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

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let raw_value = parts[1].trim().to_string();
                let value = Self::resolve_env_vars(&raw_value);
                values.insert(key, value);
            }
        }

        Ok(Config { values })
    }

    fn resolve_env_vars(value: &str) -> String {
        let mut result = value.to_string();
        for (key, env_value) in env::vars() {
            let placeholder = format!("${{{}}}", key);
            if result.contains(&placeholder) {
                result = result.replace(&placeholder, &env_value);
            }
        }
        result
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
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost:5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "MAX_CONNECTIONS=100").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost:5432");
        assert_eq!(config.get("MAX_CONNECTIONS").unwrap(), "100");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_var_substitution() {
        env::set_var("APP_PORT", "8080");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PORT=${{APP_PORT}}").unwrap();
        writeln!(file, "HOST=localhost:${{APP_PORT}}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("HOST").unwrap(), "localhost:8080");
    }
}