
use serde_json::Value;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    JsonError(serde_json::Error),
    InvalidLogFormat,
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, ParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let entry = parse_log_line(&line)?;
        entries.push(entry);
    }

    Ok(entries)
}

fn parse_log_line(line: &str) -> Result<LogEntry, ParseError> {
    let json_value: Value = serde_json::from_str(line)?;

    let timestamp = json_value["timestamp"]
        .as_str()
        .ok_or(ParseError::InvalidLogFormat)?
        .to_string();

    let level = json_value["level"]
        .as_str()
        .ok_or(ParseError::InvalidLogFormat)?
        .to_string();

    let message = json_value["message"]
        .as_str()
        .ok_or(ParseError::InvalidLogFormat)?
        .to_string();

    let metadata = json_value["metadata"].clone();

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
        .filter(|entry| entry.level.eq_ignore_ascii_case(level))
        .collect()
}

pub fn extract_timestamps(entries: &[LogEntry]) -> Vec<String> {
    entries.iter().map(|entry| entry.timestamp.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Service started","metadata":{"pid":1234}}"#;
        let entry = parse_log_line(log_data).unwrap();

        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "Service started");
        assert_eq!(entry.metadata["pid"], 1234);
    }

    #[test]
    fn test_filter_logs() {
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Service started".to_string(),
                metadata: json!({}),
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Connection failed".to_string(),
                metadata: json!({}),
            },
        ];

        let errors = filter_by_level(&entries, "ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Connection failed");
    }

    #[test]
    fn test_parse_log_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_lines = vec![
            r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Start","metadata":{}}"#,
            r#"{"timestamp":"2024-01-15T10:31:00Z","level":"WARN","message":"Slow response","metadata":{"latency":1500}}"#,
        ];

        for line in log_lines {
            writeln!(temp_file, "{}", line).unwrap();
        }

        let entries = parse_log_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].level, "WARN");
    }
}