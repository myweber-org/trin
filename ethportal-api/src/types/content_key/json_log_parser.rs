
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    fields: HashMap<String, Value>,
}

pub struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                self.entries.push(entry);
                count += 1;
            }
        }

        Ok(count)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let timestamp = json_value.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_string();

        let message = json_value.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
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

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    pub fn search_messages(&self, query: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|entry| entry.message.contains(query))
            .collect()
    }

    pub fn summarize(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        
        for entry in &self.entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        summary
    }

    pub fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parser_creation() {
        let parser = LogParser::new();
        assert_eq!(parser.get_entries().len(), 0);
    }

    #[test]
    fn test_summary() {
        let mut parser = LogParser::new();
        
        let test_logs = vec![
            json!({"timestamp": "2024-01-01T10:00:00Z", "level": "INFO", "message": "System started"}).to_string(),
            json!({"timestamp": "2024-01-01T10:01:00Z", "level": "ERROR", "message": "Connection failed", "error_code": 500}).to_string(),
            json!({"timestamp": "2024-01-01T10:02:00Z", "level": "INFO", "message": "Retrying connection"}).to_string(),
        ];

        for log in test_logs {
            let _ = parser.parse_line(&log);
        }

        let summary = parser.summarize();
        assert_eq!(summary.get("INFO"), Some(&2));
        assert_eq!(summary.get("ERROR"), Some(&1));
    }
}