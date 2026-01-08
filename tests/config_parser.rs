use std::collections::HashMap;
use std::env;
use regex::Regex;

pub struct ConfigParser {
    values: HashMap<String, String>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            values: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        self.parse_content(&content)
    }

    pub fn parse_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let var_regex = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")?;
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let mut processed_value = value.trim().to_string();

                for capture in var_regex.captures_iter(&processed_value) {
                    if let Some(var_name) = capture.get(1) {
                        if let Ok(env_value) = env::var(var_name.as_str()) {
                            processed_value = processed_value.replace(&capture[0], &env_value);
                        }
                    }
                }

                self.values.insert(key, processed_value);
            }
        }
        Ok(())
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
        let mut parser = ConfigParser::new();
        let content = "DATABASE_HOST=localhost\nDATABASE_PORT=5432\n";
        parser.parse_content(content).unwrap();

        assert_eq!(parser.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(parser.get("DATABASE_PORT"), Some(&"5432".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "super_secret_key");
        
        let mut parser = ConfigParser::new();
        let content = "SECRET_KEY=${APP_SECRET}\nAPI_KEY=static_value";
        parser.parse_content(content).unwrap();

        assert_eq!(parser.get("SECRET_KEY"), Some(&"super_secret_key".to_string()));
        assert_eq!(parser.get("API_KEY"), Some(&"static_value".to_string()));
    }

    #[test]
    fn test_file_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "SERVER_HOST=127.0.0.1").unwrap();
        writeln!(temp_file, "SERVER_PORT=8080").unwrap();

        let mut parser = ConfigParser::new();
        parser.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(parser.get("SERVER_HOST"), Some(&"127.0.0.1".to_string()));
        assert_eq!(parser.get("SERVER_PORT"), Some(&"8080".to_string()));
    }
}