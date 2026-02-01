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
    required_fields: Vec<String>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            required_fields: Vec::new(),
        }
    }

    pub fn with_required_fields(mut self, fields: &[&str]) -> Self {
        self.required_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            if let Some(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let json_value: Value = serde_json::from_str(line).ok()?;
        
        let obj = json_value.as_object()?;
        
        let timestamp = obj.get("timestamp")
            .and_then(Value::as_str)
            .map(String::from)
            .unwrap_or_else(|| "unknown".to_string());
        
        let level = obj.get("level")
            .and_then(Value::as_str)
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "info".to_string());
        
        if !self.is_level_allowed(&level) {
            return None;
        }
        
        let message = obj.get("message")
            .and_then(Value::as_str)
            .map(String::from)
            .unwrap_or_else(|| "".to_string());
        
        let mut fields = HashMap::new();
        
        for (key, value) in obj {
            if key != "timestamp" && key != "level" && key != "message" {
                fields.insert(key.clone(), value.clone());
            }
        }
        
        if !self.required_fields.is_empty() {
            for field in &self.required_fields {
                if !fields.contains_key(field) && !obj.contains_key(field) {
                    return None;
                }
            }
        }
        
        Some(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn is_level_allowed(&self, level: &str) -> bool {
        let level_order = ["trace", "debug", "info", "warn", "error", "fatal"];
        
        let min_index = level_order.iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(2);
        
        let entry_index = level_order.iter()
            .position(|&l| l == level)
            .unwrap_or(min_index);
        
        entry_index >= min_index
    }

    pub fn extract_field_values(&self, entries: &[LogEntry], field_name: &str) -> Vec<Value> {
        entries.iter()
            .filter_map(|entry| entry.fields.get(field_name))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parser_filters_by_level() {
        let parser = LogParser::new("warn");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01T12:00:00Z", "level": "INFO", "message": "Test info"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01T12:00:01Z", "level": "WARN", "message": "Test warn"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01T12:00:02Z", "level": "ERROR", "message": "Test error"}}"#).unwrap();
        
        let entries = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "warn");
        assert_eq!(entries[1].level, "error");
    }

    #[test]
    fn test_extract_field_values() {
        let parser = LogParser::new("info");
        
        let mut entries = Vec::new();
        let mut fields1 = HashMap::new();
        fields1.insert("user_id".to_string(), Value::Number(123.into()));
        fields1.insert("action".to_string(), Value::String("login".to_string()));
        
        let mut fields2 = HashMap::new();
        fields2.insert("user_id".to_string(), Value::Number(456.into()));
        fields2.insert("action".to_string(), Value::String("logout".to_string()));
        
        entries.push(LogEntry {
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            level: "info".to_string(),
            message: "User action".to_string(),
            fields: fields1,
        });
        
        entries.push(LogEntry {
            timestamp: "2024-01-01T12:00:01Z".to_string(),
            level: "info".to_string(),
            message: "User action".to_string(),
            fields: fields2,
        });
        
        let user_ids = parser.extract_field_values(&entries, "user_id");
        assert_eq!(user_ids.len(), 2);
        
        if let Value::Number(num) = &user_ids[0] {
            assert_eq!(num.as_i64(), Some(123));
        }
    }
}