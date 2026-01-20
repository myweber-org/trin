use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
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

    pub fn set_level_filter(&mut self, level: &str) -> &mut Self {
        self.filter_level = Some(level.to_lowercase());
        self
    }

    pub fn add_required_field(&mut self, field: &str) -> &mut Self {
        self.required_fields.push(field.to_string());
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| e.to_string())?;
            
            if let Some(entry) = self.parse_line(&line) {
                entries.push(entry);
            } else {
                eprintln!("Warning: Failed to parse line {}", line_num + 1);
            }
        }
        
        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let json_value: Value = serde_json::from_str(line).ok()?;
        
        let mut fields = HashMap::new();
        let mut timestamp = None;
        let mut level = None;
        let mut message = None;
        
        if let Value::Object(map) = json_value {
            for (key, value) in map {
                match key.as_str() {
                    "timestamp" | "time" | "@timestamp" => {
                        timestamp = value.as_str().map(|s| s.to_string());
                    }
                    "level" | "severity" | "log.level" => {
                        level = value.as_str().map(|s| s.to_lowercase());
                    }
                    "message" | "msg" | "log.message" => {
                        message = value.as_str().map(|s| s.to_string());
                    }
                    _ => {
                        fields.insert(key, value);
                    }
                }
            }
        }
        
        if let Some(filter) = &self.filter_level {
            if let Some(entry_level) = &level {
                if entry_level != filter {
                    return None;
                }
            } else {
                return None;
            }
        }
        
        for required_field in &self.required_fields {
            if !fields.contains_key(required_field) {
                return None;
            }
        }
        
        Some(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn extract_field_values(&self, entries: &[LogEntry], field_name: &str) -> Vec<Value> {
        entries
            .iter()
            .filter_map(|entry| entry.fields.get(field_name).cloned())
            .collect()
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
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_json_log() {
        let mut parser = LogParser::new();
        parser.set_level_filter("error");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp": "2023-10-01T12:00:00Z", "level": "error", "message": "Database connection failed", "error_code": 500}
{"timestamp": "2023-10-01T12:01:00Z", "level": "info", "message": "Server started", "port": 8080}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let entries = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        
        let entry = &entries[0];
        assert_eq!(entry.level.as_deref(), Some("error"));
        assert_eq!(entry.message.as_deref(), Some("Database connection failed"));
        assert_eq!(entry.fields.get("error_code"), Some(&json!(500)));
    }

    #[test]
    fn test_extract_field_values() {
        let parser = LogParser::new();
        
        let mut entry1 = LogEntry {
            timestamp: Some("2023-10-01T12:00:00Z".to_string()),
            level: Some("error".to_string()),
            message: Some("Error 1".to_string()),
            fields: HashMap::from([("code".to_string(), json!(100))]),
        };
        
        let mut entry2 = LogEntry {
            timestamp: Some("2023-10-01T12:01:00Z".to_string()),
            level: Some("error".to_string()),
            message: Some("Error 2".to_string()),
            fields: HashMap::from([("code".to_string(), json!(200))]),
        };
        
        let entries = vec![entry1, entry2];
        let codes = parser.extract_field_values(&entries, "code");
        
        assert_eq!(codes, vec![json!(100), json!(200)]);
    }
}