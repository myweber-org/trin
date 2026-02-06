
use std::collections::HashMap;
use std::fs;

#[derive(Debug, PartialEq)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Table(HashMap<String, ConfigValue>),
}

#[derive(Debug)]
pub struct ConfigParser {
    data: HashMap<String, ConfigValue>,
}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {
            data: HashMap::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut parser = ConfigParser::new();
        parser.parse(&content)?;
        Ok(parser)
    }

    pub fn parse(&mut self, content: &str) -> Result<(), String> {
        let mut current_section = String::new();
        
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len()-1].to_string();
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid syntax at line {}", line_num + 1));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            
            let full_key = if current_section.is_empty() {
                key
            } else {
                format!("{}.{}", current_section, key)
            };

            self.data.insert(full_key, Self::parse_value(&value));
        }
        
        Ok(())
    }

    fn parse_value(value: &str) -> ConfigValue {
        if value.starts_with('"') && value.ends_with('"') {
            ConfigValue::String(value[1..value.len()-1].to_string())
        } else if value == "true" {
            ConfigValue::Boolean(true)
        } else if value == "false" {
            ConfigValue::Boolean(false)
        } else if value.contains('.') {
            if let Ok(num) = value.parse::<f64>() {
                ConfigValue::Float(num)
            } else {
                ConfigValue::String(value.to_string())
            }
        } else if let Ok(num) = value.parse::<i64>() {
            ConfigValue::Integer(num)
        } else {
            ConfigValue::String(value.to_string())
        }
    }

    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.data.get(key)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.get(key) {
            Some(ConfigValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn get_integer(&self, key: &str) -> Option<i64> {
        match self.get(key) {
            Some(ConfigValue::Integer(i)) => Some(*i),
            _ => None,
        }
    }

    pub fn validate_required(&self, keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();
        
        for key in keys {
            if !self.data.contains_key(*key) {
                missing.push(key.to_string());
            }
        }
        
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let content = r#"
            name = "test_app"
            version = 1
            enabled = true
            timeout = 30.5
            
            [database]
            host = "localhost"
            port = 5432
        "#;

        let mut parser = ConfigParser::new();
        parser.parse(content).unwrap();

        assert_eq!(parser.get_string("name"), Some("test_app".to_string()));
        assert_eq!(parser.get_integer("version"), Some(1));
        assert_eq!(parser.get_string("database.host"), Some("localhost".to_string()));
        assert_eq!(parser.get_integer("database.port"), Some(5432));
    }

    #[test]
    fn test_validation() {
        let content = "key1 = value1\nkey2 = value2";
        let mut parser = ConfigParser::new();
        parser.parse(content).unwrap();

        let result = parser.validate_required(&["key1", "key2", "missing_key"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec!["missing_key".to_string()]);
    }
}