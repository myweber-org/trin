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

    fn process_value(raw: &str) -> String {
        let mut result = String::new();
        let mut chars = raw.chars().peekable();
        let mut in_braces = false;
        let mut var_name = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '$' if chars.peek() == Some(&'{') => {
                    chars.next();
                    in_braces = true;
                    var_name.clear();
                }
                '}' if in_braces => {
                    in_braces = false;
                    if let Ok(env_value) = env::var(&var_name) {
                        result.push_str(&env_value);
                    } else {
                        result.push_str(&format!("${{{}}}", var_name));
                    }
                }
                _ if in_braces => {
                    var_name.push(ch);
                }
                _ => {
                    result.push(ch);
                }
            }
        }

        if in_braces {
            result.push_str(&format!("${{{}}}", var_name));
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
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "ENVIRONMENT=production").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("ENVIRONMENT"), Some(&"production".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "mysecret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET=${{APP_SECRET}}").unwrap();
        writeln!(file, "PATH=/home/${{USER}}/data").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET"), Some(&"mysecret123".to_string()));
        
        if let Ok(user) = env::var("USER") {
            assert_eq!(config.get("PATH"), Some(&format!("/home/{}/data", user)));
        }
    }

    #[test]
    fn test_missing_env_var() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "MISSING=${{NONEXISTENT_VAR}}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("MISSING"), Some(&"${NONEXISTENT_VAR}".to_string()));
    }
}