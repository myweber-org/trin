use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidFormat(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

pub struct LogParser {
    min_level: String,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)?;
            if self.should_include(&entry.level) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn should_include(&self, level: &str) -> bool {
        let level_order = ["trace", "debug", "info", "warn", "error", "fatal"];
        let min_index = level_order
            .iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(0);
        let entry_index = level_order
            .iter()
            .position(|&l| l == level.to_lowercase())
            .unwrap_or(level_order.len());

        entry_index >= min_index
    }

    pub fn filter_by_level(&self, entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
        entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn extract_messages(&self, entries: &[LogEntry]) -> Vec<String> {
        entries
            .iter()
            .map(|entry| format!("[{}] {}", entry.level, entry.message))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_logs() {
        let logs = r#"{"timestamp": "2024-01-01T12:00:00Z", "level": "INFO", "message": "System started"}
{"timestamp": "2024-01-01T12:01:00Z", "level": "ERROR", "message": "Connection failed", "metadata": {"retry_count": 3}}"#;

        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), logs).unwrap();

        let parser = LogParser::new("info");
        let result = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].level, "INFO");
        assert_eq!(result[1].message, "Connection failed");
    }

    #[test]
    fn test_level_filtering() {
        let parser = LogParser::new("warn");
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-01T12:00:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Info message".to_string(),
                metadata: serde_json::Value::Null,
            },
            LogEntry {
                timestamp: "2024-01-01T12:01:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Error message".to_string(),
                metadata: serde_json::Value::Null,
            },
        ];

        let filtered = parser.filter_by_level(&entries, "error");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].level, "ERROR");
    }
}