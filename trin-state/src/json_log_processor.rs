use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    InvalidFormat(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<serde_json::Error> for LogError {
    fn from(err: serde_json::Error) -> Self {
        LogError::ParseError(err)
    }
}

pub fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, LogError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        match parse_log_line(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Warning: Failed to parse line {}: {:?}", line_num + 1, e),
        }
    }

    Ok(entries)
}

fn parse_log_line(line: &str) -> Result<LogEntry, LogError> {
    let json_value: Value = serde_json::from_str(line)?;

    let timestamp = json_value["timestamp"]
        .as_str()
        .ok_or_else(|| LogError::InvalidFormat("Missing timestamp".to_string()))?
        .to_string();

    let level = json_value["level"]
        .as_str()
        .ok_or_else(|| LogError::InvalidFormat("Missing level".to_string()))?
        .to_string();

    let message = json_value["message"]
        .as_str()
        .ok_or_else(|| LogError::InvalidFormat("Missing message".to_string()))?
        .to_string();

    let metadata = json_value.get("metadata").cloned().unwrap_or(Value::Null);

    Ok(LogEntry {
        timestamp,
        level,
        message,
        metadata,
    })
}

pub fn filter_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_log_line() {
        let json_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","metadata":{"attempt":3}}"#;
        let entry = parse_log_line(json_line).unwrap();
        
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.metadata["attempt"], 3);
    }

    #[test]
    fn test_parse_invalid_json() {
        let invalid_line = r#"{"timestamp":"2024-01-15","level":"INFO"#;
        let result = parse_log_line(invalid_line);
        assert!(matches!(result, Err(LogError::ParseError(_))));
    }
}