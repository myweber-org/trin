use std::collections::HashMap;
use std::fs;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value for field {field}: {value}")]
    InvalidValue { field: String, value: String },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub sections: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, ConfigError> {
        let mut settings = HashMap::new();
        let mut sections = HashMap::new();
        let mut current_section = None;

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                let section_name = line[1..line.len()-1].trim().to_string();
                if section_name.is_empty() {
                    return Err(ConfigError::ParseError {
                        line: line_num + 1,
                        message: "Empty section name".to_string(),
                    });
                }
                current_section = Some(section_name.clone());
                sections.insert(section_name, HashMap::new());
                continue;
            }

            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos+1..].trim().to_string();

                if key.is_empty() {
                    return Err(ConfigError::ParseError {
                        line: line_num + 1,
                        message: "Empty key before '='".to_string(),
                    });
                }

                match &current_section {
                    Some(section) => {
                        sections.get_mut(section)
                            .ok_or_else(|| ConfigError::ParseError {
                                line: line_num + 1,
                                message: format!("Section '{}' not found", section),
                            })?
                            .insert(key, value);
                    }
                    None => {
                        settings.insert(key, value);
                    }
                }
            } else {
                return Err(ConfigError::ParseError {
                    line: line_num + 1,
                    message: "Line must contain '=' separator".to_string(),
                });
            }
        }

        Ok(Config { settings, sections })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_from_section(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section).and_then(|s| s.get(key))
    }

    pub fn require(&self, key: &str) -> Result<&String, ConfigError> {
        self.get(key).ok_or_else(|| ConfigError::MissingField(key.to_string()))
    }

    pub fn require_from_section(&self, section: &str, key: &str) -> Result<&String, ConfigError> {
        self.get_from_section(section, key)
            .ok_or_else(|| ConfigError::MissingField(format!("{}.{}", section, key)))
    }

    pub fn validate_port(&self, key: &str) -> Result<u16, ConfigError> {
        let value = self.require(key)?;
        value.parse::<u16>()
            .map_err(|_| ConfigError::InvalidValue {
                field: key.to_string(),
                value: value.clone(),
            })
    }

    pub fn validate_bool(&self, key: &str) -> Result<bool, ConfigError> {
        let value = self.require(key)?;
        match value.to_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            _ => Err(ConfigError::InvalidValue {
                field: key.to_string(),
                value: value.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let config = Config::parse("
            server_host = localhost
            server_port = 8080
            enable_logging = true
            
            [database]
            host = db.local
            port = 5432
        ").unwrap();

        assert_eq!(config.get("server_host").unwrap(), "localhost");
        assert_eq!(config.get_from_section("database", "host").unwrap(), "db.local");
        assert_eq!(config.validate_port("server_port").unwrap(), 8080);
        assert_eq!(config.validate_bool("enable_logging").unwrap(), true);
    }

    #[test]
    fn test_missing_field() {
        let config = Config::parse("key1 = value1").unwrap();
        assert!(config.require("nonexistent").is_err());
    }

    #[test]
    fn test_invalid_syntax() {
        assert!(Config::parse("invalid_line_without_equals").is_err());
        assert!(Config::parse("=novalue").is_err());
        assert!(Config::parse("[]").is_err());
    }
}