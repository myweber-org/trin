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
        let var_pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, mut value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                
                for cap in var_pattern.captures_iter(&value) {
                    if let Some(var_name) = cap.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            value = value.replace(&cap[0], &env_value);
                        }
                    }
                }
                
                values.insert(key, value.trim().to_string());
            }
        }

        Ok(Config { values })
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
        let content = "host=localhost\nport=8080\n# comment\n\ndebug=true";
        let config = Config::from_str(content).unwrap();
        
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("debug"), Some(&"true".to_string()));
        assert_eq!(config.get("missing"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "9090");
        
        let content = "host=localhost\nport=${APP_PORT}\nurl=http://${host}:${APP_PORT}";
        let config = Config::from_str(content).unwrap();
        
        assert_eq!(config.get("port"), Some(&"9090".to_string()));
        assert_eq!(config.get("url"), Some(&"http://localhost:9090".to_string()));
    }

    #[test]
    fn test_file_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "database=postgresql").unwrap();
        writeln!(file, "timeout=30").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("database"), Some(&"postgresql".to_string()));
        assert_eq!(config.get_or_default("timeout", "10"), "30");
        assert_eq!(config.get_or_default("missing", "default_value"), "default_value");
    }
}