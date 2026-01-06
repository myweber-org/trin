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
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
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
        env::set_var("APP_SECRET", "mysecret123");
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET=${{APP_SECRET}}").unwrap();
        writeln!(file, "PATH=/home/${{USER}}/data").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET"), Some(&"mysecret123".to_string()));
    }
}use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    sections: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            sections: HashMap::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, String> {
        let mut config = Config::new();
        let mut current_section = String::from("default");
        let mut line_number = 0;

        for line in content.lines() {
            line_number += 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].trim().to_string();
                if current_section.is_empty() {
                    return Err(format!("Empty section name at line {}", line_number));
                }
                config.sections.entry(current_section.clone()).or_insert_with(HashMap::new);
                continue;
            }

            if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let mut value = trimmed[equal_pos + 1..].trim().to_string();

                if key.is_empty() {
                    return Err(format!("Empty key at line {}", line_number));
                }

                value = Self::substitute_env_vars(&value);

                config
                    .sections
                    .entry(current_section.clone())
                    .or_insert_with(HashMap::new)
                    .insert(key, value);
            } else {
                return Err(format!("Invalid line format at line {}", line_number));
            }
        }

        Ok(config)
    }

    fn substitute_env_vars(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        let mut in_var = false;
        let mut var_name = String::new();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next();
                in_var = true;
                var_name.clear();
                continue;
            }

            if in_var && ch == '}' {
                in_var = false;
                let env_value = env::var(&var_name).unwrap_or_else(|_| String::new());
                result.push_str(&env_value);
                var_name.clear();
                continue;
            }

            if in_var {
                var_name.push(ch);
            } else {
                result.push(ch);
            }
        }

        result
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section)?.get(key)
    }

    pub fn get_with_default(&self, section: &str, key: &str, default: &str) -> String {
        self.get(section, key).cloned().unwrap_or_else(|| default.to_string())
    }

    pub fn sections(&self) -> Vec<&String> {
        self.sections.keys().collect()
    }

    pub fn keys(&self, section: &str) -> Option<Vec<&String>> {
        self.sections.get(section).map(|map| map.keys().collect())
    }

    pub fn merge(&mut self, other: Config) {
        for (section, kv_pairs) in other.sections {
            let section_map = self.sections.entry(section).or_insert_with(HashMap::new);
            for (key, value) in kv_pairs {
                section_map.insert(key, value);
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
[server]
host = localhost
port = 8080

[database]
url = postgresql://localhost/mydb
max_connections = 20
"#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("server", "host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("server", "port"), Some(&"8080".to_string()));
        assert_eq!(config.get("database", "url"), Some(&"postgresql://localhost/mydb".to_string()));
        assert_eq!(config.get("database", "max_connections"), Some(&"20".to_string()));
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_PORT", "3000");
        env::set_var("DB_HOST", "db.example.com");

        let content = r#"
[app]
port = ${APP_PORT}
host = ${DB_HOST}
missing = ${NON_EXISTENT}
"#;

        let config = Config::from_str(content).unwrap();
        assert_eq!(config.get("app", "port"), Some(&"3000".to_string()));
        assert_eq!(config.get("app", "host"), Some(&"db.example.com".to_string()));
        assert_eq!(config.get("app", "missing"), Some(&"".to_string()));
    }

    #[test]
    fn test_merge() {
        let mut config1 = Config::from_str("[section1]\nkey1 = value1").unwrap();
        let config2 = Config::from_str("[section1]\nkey2 = value2\n[section2]\nkey3 = value3").unwrap();

        config1.merge(config2);
        assert_eq!(config1.get("section1", "key1"), Some(&"value1".to_string()));
        assert_eq!(config1.get("section1", "key2"), Some(&"value2".to_string()));
        assert_eq!(config1.get("section2", "key3"), Some(&"value3".to_string()));
    }
}