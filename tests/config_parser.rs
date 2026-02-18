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
}use std::collections::HashMap;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    sections: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            sections: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        let mut config = Config::new();
        let mut current_section = String::from("default");
        config.sections.insert(current_section.clone(), HashMap::new());

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section_name = trimmed[1..trimmed.len() - 1].trim().to_string();
                if section_name.is_empty() {
                    return Err(format!("Invalid section name at line {}", line_num + 1));
                }
                current_section = section_name;
                config.sections.entry(current_section.clone()).or_insert_with(HashMap::new);
            } else if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let value = trimmed[equal_pos + 1..].trim().to_string();
                if key.is_empty() {
                    return Err(format!("Empty key at line {}", line_num + 1));
                }
                config.sections
                    .entry(current_section.clone())
                    .or_insert_with(HashMap::new)
                    .insert(key, value);
            } else {
                return Err(format!("Invalid line format at line {}", line_num + 1));
            }
        }

        Ok(config)
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section)?.get(key)
    }

    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        self.sections
            .entry(section.to_string())
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
    }

    pub fn sections(&self) -> Vec<&String> {
        self.sections.keys().collect()
    }

    pub fn keys(&self, section: &str) -> Option<Vec<&String>> {
        Some(self.sections.get(section)?.keys().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let content = r#"
# Sample config
server_host = 127.0.0.1
server_port = 8080

[database]
host = localhost
port = 5432
"#;

        let config = Config::parse(content).unwrap();
        assert_eq!(config.get("default", "server_host"), Some(&"127.0.0.1".to_string()));
        assert_eq!(config.get("default", "server_port"), Some(&"8080".to_string()));
        assert_eq!(config.get("database", "host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("database", "port"), Some(&"5432".to_string()));
    }

    #[test]
    fn test_empty_section_error() {
        let content = "[]";
        let result = Config::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_and_get() {
        let mut config = Config::new();
        config.set("default", "timeout", "30");
        assert_eq!(config.get("default", "timeout"), Some(&"30".to_string()));
    }
}