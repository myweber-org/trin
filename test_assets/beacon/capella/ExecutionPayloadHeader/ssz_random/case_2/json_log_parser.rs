use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: String,
    filter_fields: Vec<String>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            filter_fields: Vec::new(),
        }
    }

    pub fn add_filter_field(&mut self, field: &str) {
        self.filter_fields.push(field.to_string());
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

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_lowercase();

        if !self.is_level_allowed(&level) {
            return Err("Log level below threshold".into());
        }

        let timestamp = json_value.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let message = json_value.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if !self.filter_fields.contains(key) {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn is_level_allowed(&self, level: &str) -> bool {
        let level_order = vec!["trace", "debug", "info", "warn", "error", "fatal"];
        
        let min_index = level_order.iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(0);
        
        let current_index = level_order.iter()
            .position(|&l| l == level)
            .unwrap_or(0);

        current_index >= min_index
    }

    pub fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = format!("[{}] {}: {}", 
            entry.timestamp, 
            entry.level.to_uppercase(), 
            entry.message
        );

        if !entry.fields.is_empty() {
            output.push_str(" | ");
            for (key, value) in &entry.fields {
                output.push_str(&format!("{}={:?} ", key, value));
            }
        }

        output.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_parsing() {
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"System started","user":"admin","session_id":"abc123"}"#;
        
        let parser = LogParser::new("info");
        let entry = parser.parse_line(log_line).unwrap();
        
        assert_eq!(entry.level, "info");
        assert_eq!(entry.message, "System started");
        assert_eq!(entry.fields.len(), 2);
    }

    #[test]
    fn test_level_filtering() {
        let parser = LogParser::new("warn");
        
        assert!(parser.is_level_allowed("warn"));
        assert!(parser.is_level_allowed("error"));
        assert!(!parser.is_level_allowed("info"));
    }
}