
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid log format at line {0}")]
    InvalidFormat(usize),
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct JsonLogParser {
    file_path: String,
}

impl JsonLogParser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, LogParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let line_number = line_num + 1;

            if line.trim().is_empty() {
                continue;
            }

            let parsed: Value = serde_json::from_str(&line)
                .map_err(|e| LogParseError::Json(e))?;

            let entry = self.parse_json_value(&parsed, line_number)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    fn parse_json_value(&self, value: &Value, line_number: usize) -> Result<LogEntry, LogParseError> {
        let obj = value.as_object()
            .ok_or_else(|| LogParseError::InvalidFormat(line_number))?;

        let timestamp = obj.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LogParseError::InvalidFormat(line_number))?
            .to_string();

        let level = obj.get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LogParseError::InvalidFormat(line_number))?
            .to_string();

        let message = obj.get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LogParseError::InvalidFormat(line_number))?
            .to_string();

        let mut fields = HashMap::new();
        for (key, value) in obj.iter() {
            if !["timestamp", "level", "message"].contains(&key.as_str()) {
                fields.insert(key.clone(), value.clone());
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, LogParseError> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect();
        Ok(filtered)
    }

    pub fn count_entries(&self) -> Result<usize, LogParseError> {
        let entries = self.parse()?;
        Ok(entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "Service started", "service": "api"}}"#).unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "message": "Connection failed", "error_code": 500}}"#).unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:32:00Z", "level": "WARN", "message": "High memory usage", "usage_percent": 85}}"#).unwrap();
        file
    }

    #[test]
    fn test_parse_valid_log() {
        let test_file = create_test_log();
        let parser = JsonLogParser::new(test_file.path().to_str().unwrap());
        let result = parser.parse();
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_filter_by_level() {
        let test_file = create_test_log();
        let parser = JsonLogParser::new(test_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Connection failed");
    }

    #[test]
    fn test_count_entries() {
        let test_file = create_test_log();
        let parser = JsonLogParser::new(test_file.path().to_str().unwrap());
        let count = parser.count_entries().unwrap();
        assert_eq!(count, 3);
    }
}