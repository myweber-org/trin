use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: String,
    required_fields: Vec<String>,
}

impl LogParser {
    pub fn new(min_level: &str, required_fields: Vec<&str>) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            required_fields: required_fields.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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
        let json: Value = serde_json::from_str(line)?;
        
        let level = json.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_lowercase();

        if !self.is_level_allowed(&level) {
            return Err("Log level below threshold".into());
        }

        let mut fields = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                fields.insert(key.clone(), value.clone());
            }
        }

        let timestamp = fields.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let message = fields.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let entry = LogEntry {
            timestamp,
            level,
            message,
            fields,
        };

        if self.has_required_fields(&entry) {
            Ok(entry)
        } else {
            Err("Missing required fields".into())
        }
    }

    fn is_level_allowed(&self, level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error", "fatal"];
        let min_idx = levels.iter().position(|&l| l == self.min_level).unwrap_or(0);
        let entry_idx = levels.iter().position(|&l| l == level).unwrap_or(0);
        entry_idx >= min_idx
    }

    fn has_required_fields(&self, entry: &LogEntry) -> bool {
        self.required_fields.iter()
            .all(|field| entry.fields.contains_key(field))
    }

    pub fn extract_field_values(&self, entries: &[LogEntry], field: &str) -> Vec<Value> {
        entries.iter()
            .filter_map(|entry| entry.fields.get(field))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_filters_by_level() {
        let parser = LogParser::new("warn", vec![]);
        let test_json = r#"{"level": "info", "message": "test"}"#;
        
        assert!(parser.parse_line(test_json).is_err());
    }

    #[test]
    fn test_parser_requires_fields() {
        let parser = LogParser::new("info", vec!["user_id"]);
        let test_json = r#"{"level": "info", "message": "test"}"#;
        
        assert!(parser.parse_line(test_json).is_err());
    }
}