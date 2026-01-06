use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub extra_fields: Vec<(String, Value)>,
}

pub struct LogParser {
    required_fields: Vec<String>,
}

impl LogParser {
    pub fn new(required_fields: Vec<&str>) -> Self {
        LogParser {
            required_fields: required_fields.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn parse_file(&self, path: &Path) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            match self.parse_line(&line) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => continue,
                Err(e) => eprintln!("Line {} parse error: {}", line_num + 1, e),
            }
        }
        
        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<Option<LogEntry>, String> {
        let json_value: Value = serde_json::from_str(line)
            .map_err(|e| format!("Invalid JSON: {}", e))?;

        let obj = json_value.as_object()
            .ok_or_else(|| "Expected JSON object".to_string())?;

        let mut missing_fields = Vec::new();
        for field in &self.required_fields {
            if !obj.contains_key(field) {
                missing_fields.push(field.clone());
            }
        }

        if !missing_fields.is_empty() {
            return Err(format!("Missing required fields: {:?}", missing_fields));
        }

        let timestamp = obj.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid timestamp".to_string())?
            .to_string();

        let level = obj.get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid level".to_string())?
            .to_string();

        let message = obj.get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid message".to_string())?
            .to_string();

        let mut extra_fields = Vec::new();
        for (key, value) in obj {
            if !self.required_fields.contains(key) && key != "timestamp" && key != "level" && key != "message" {
                extra_fields.push((key.clone(), value.clone()));
            }
        }

        Ok(Some(LogEntry {
            timestamp,
            level,
            message,
            extra_fields,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parser_with_valid_json() {
        let parser = LogParser::new(vec!["timestamp", "level", "message"]);
        let json_line = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","message":"System started","user":"admin"}"#;
        
        let result = parser.parse_line(json_line).unwrap().unwrap();
        assert_eq!(result.timestamp, "2023-10-01T12:00:00Z");
        assert_eq!(result.level, "INFO");
        assert_eq!(result.message, "System started");
        assert_eq!(result.extra_fields.len(), 1);
        assert_eq!(result.extra_fields[0].0, "user");
    }

    #[test]
    fn test_parser_missing_required_field() {
        let parser = LogParser::new(vec!["timestamp", "level", "message"]);
        let json_line = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO"}"#;
        
        let result = parser.parse_line(json_line);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required fields"));
    }

    #[test]
    fn test_parse_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","message":"Line 1"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2023-10-01T12:01:00Z","level":"ERROR","message":"Line 2","error_code":500}}"#).unwrap();
        
        let parser = LogParser::new(vec!["timestamp", "level", "message"]);
        let entries = parser.parse_file(temp_file.path()).unwrap();
        
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].level, "ERROR");
        assert_eq!(entries[1].extra_fields.len(), 1);
    }
}