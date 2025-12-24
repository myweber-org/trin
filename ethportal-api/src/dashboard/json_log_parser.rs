use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub message: Option<String>,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    filter_level: Option<String>,
    required_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filter_level: None,
            required_fields: Vec::new(),
        }
    }

    pub fn set_level_filter(&mut self, level: &str) {
        self.filter_level = Some(level.to_lowercase());
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.push(field.to_string());
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    pub fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let mut entry = LogEntry {
            timestamp: json_value.get("timestamp").and_then(|v| v.as_str()).map(|s| s.to_string()),
            level: json_value.get("level").and_then(|v| v.as_str()).map(|s| s.to_lowercase()),
            message: json_value.get("message").and_then(|v| v.as_str()).map(|s| s.to_string()),
            fields: HashMap::new(),
        };

        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if !["timestamp", "level", "message"].contains(&key.as_str()) {
                    entry.fields.insert(key.clone(), value.clone());
                }
            }
        }

        if let Some(filter) = &self.filter_level {
            if let Some(level) = &entry.level {
                if level != filter {
                    return Err("Log level filtered out".into());
                }
            }
        }

        for field in &self.required_fields {
            if !entry.fields.contains_key(field) && 
               !entry.timestamp.as_ref().map(|_| field == "timestamp").unwrap_or(false) &&
               !entry.level.as_ref().map(|_| field == "level").unwrap_or(false) &&
               !entry.message.as_ref().map(|_| field == "message").unwrap_or(false) {
                return Err(format!("Required field '{}' not found", field).into());
            }
        }

        Ok(entry)
    }

    pub fn extract_field_values(&self, entries: &[LogEntry], field_name: &str) -> Vec<Value> {
        let mut values = Vec::new();
        
        for entry in entries {
            if let Some(value) = entry.fields.get(field_name) {
                values.push(value.clone());
            } else if field_name == "timestamp" {
                if let Some(ts) = &entry.timestamp {
                    values.push(Value::String(ts.clone()));
                }
            } else if field_name == "level" {
                if let Some(lvl) = &entry.level {
                    values.push(Value::String(lvl.clone()));
                }
            } else if field_name == "message" {
                if let Some(msg) = &entry.message {
                    values.push(Value::String(msg.clone()));
                }
            }
        }
        
        values
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json_log() {
        let parser = LogParser::new();
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"error","message":"Connection failed","user_id":123,"ip":"192.168.1.1"}"#;
        
        let result = parser.parse_line(log_line);
        assert!(result.is_ok());
        
        let entry = result.unwrap();
        assert_eq!(entry.timestamp, Some("2024-01-15T10:30:00Z".to_string()));
        assert_eq!(entry.level, Some("error".to_string()));
        assert_eq!(entry.message, Some("Connection failed".to_string()));
        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields.get("user_id").and_then(|v| v.as_i64()), Some(123));
    }

    #[test]
    fn test_level_filter() {
        let mut parser = LogParser::new();
        parser.set_level_filter("error");
        
        let error_log = r#"{"level":"error","message":"Error occurred"}"#;
        let info_log = r#"{"level":"info","message":"Info message"}"#;
        
        assert!(parser.parse_line(error_log).is_ok());
        assert!(parser.parse_line(info_log).is_err());
    }

    #[test]
    fn test_required_field_check() {
        let mut parser = LogParser::new();
        parser.add_required_field("user_id");
        
        let valid_log = r#"{"level":"info","message":"User action","user_id":456}"#;
        let invalid_log = r#"{"level":"info","message":"System event"}"#;
        
        assert!(parser.parse_line(valid_log).is_ok());
        assert!(parser.parse_line(invalid_log).is_err());
    }
}