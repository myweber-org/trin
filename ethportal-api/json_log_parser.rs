use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    entries: Vec<LogEntry>,
    level_filter: Option<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
            level_filter: None,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(parsed) = serde_json::from_str::<Value>(&line) {
                if let Some(entry) = self.parse_json_value(&parsed) {
                    self.entries.push(entry);
                }
            }
        }

        Ok(())
    }

    fn parse_json_value(&self, value: &Value) -> Option<LogEntry> {
        let obj = value.as_object()?;
        
        let timestamp = obj.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let level = obj.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_string();

        let message = obj.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        for (key, val) in obj {
            if !["timestamp", "level", "message"].contains(&key.as_str()) {
                fields.insert(key.clone(), val.clone());
            }
        }

        Some(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn set_level_filter(&mut self, level: &str) {
        self.level_filter = Some(level.to_lowercase());
    }

    pub fn get_filtered_entries(&self) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|entry| {
                if let Some(ref filter) = self.level_filter {
                    entry.level.to_lowercase() == *filter
                } else {
                    true
                }
            })
            .collect()
    }

    pub fn summarize(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        
        for entry in &self.entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }

        summary
    }

    pub fn find_entries_with_field(&self, field_name: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|entry| entry.fields.contains_key(field_name))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_json_value() {
        let parser = LogParser::new();
        let json_data = json!({
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "ERROR",
            "message": "Database connection failed",
            "error_code": 500,
            "service": "auth"
        });

        let entry = parser.parse_json_value(&json_data).unwrap();
        
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.fields.get("error_code").unwrap().as_i64().unwrap(), 500);
        assert_eq!(entry.fields.get("service").unwrap().as_str().unwrap(), "auth");
    }

    #[test]
    fn test_summarize() {
        let mut parser = LogParser::new();
        
        let entry1 = LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: "ERROR".to_string(),
            message: "Test error".to_string(),
            fields: HashMap::new(),
        };

        let entry2 = LogEntry {
            timestamp: "2024-01-15T10:31:00Z".to_string(),
            level: "INFO".to_string(),
            message: "Test info".to_string(),
            fields: HashMap::new(),
        };

        let entry3 = LogEntry {
            timestamp: "2024-01-15T10:32:00Z".to_string(),
            level: "ERROR".to_string(),
            message: "Another error".to_string(),
            fields: HashMap::new(),
        };

        parser.entries.push(entry1);
        parser.entries.push(entry2);
        parser.entries.push(entry3);

        let summary = parser.summarize();
        
        assert_eq!(summary.get("ERROR"), Some(&2));
        assert_eq!(summary.get("INFO"), Some(&1));
    }
}