use std::collections::HashMap;
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

    pub fn parse_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        let mut config = Config::new();
        let mut current_section = "default".to_string();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].trim().to_string();
                if current_section.is_empty() {
                    return Err(format!("Empty section name at line {}", line_num + 1));
                }
                config.sections.entry(current_section.clone()).or_default();
            } else if let Some(equal_pos) = trimmed.find('=') {
                let key = trimmed[..equal_pos].trim().to_string();
                let value = trimmed[equal_pos + 1..].trim().to_string();

                if key.is_empty() {
                    return Err(format!("Empty key at line {}", line_num + 1));
                }

                config
                    .sections
                    .entry(current_section.clone())
                    .or_default()
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

    pub fn section_exists(&self, section: &str) -> bool {
        self.sections.contains_key(section)
    }

    pub fn keys_in_section(&self, section: &str) -> Option<Vec<&String>> {
        let section_map = self.sections.get(section)?;
        Some(section_map.keys().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
# Sample config
[server]
host = 127.0.0.1
port = 8080

[database]
url = postgresql://localhost/mydb
"#;

        let config = Config::parse(content).unwrap();
        assert_eq!(config.get("server", "host"), Some(&"127.0.0.1".to_string()));
        assert_eq!(config.get("server", "port"), Some(&"8080".to_string()));
        assert_eq!(
            config.get("database", "url"),
            Some(&"postgresql://localhost/mydb".to_string())
        );
    }

    #[test]
    fn test_default_section() {
        let content = r#"
key1 = value1
key2 = value2
"#;

        let config = Config::parse(content).unwrap();
        assert_eq!(config.get("default", "key1"), Some(&"value1".to_string()));
        assert_eq!(config.get("default", "key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_invalid_line() {
        let content = "invalid line without equals";
        let result = Config::parse(content);
        assert!(result.is_err());
    }
}