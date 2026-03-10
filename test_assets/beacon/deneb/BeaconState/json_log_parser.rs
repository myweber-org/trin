
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    InvalidLevel(String),
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

pub struct LogParser {
    min_level: String,
}

impl LogParser {
    pub fn new(min_level: &str) -> Result<Self, LogError> {
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&min_level) {
            return Err(LogError::InvalidLevel(min_level.to_string()));
        }
        Ok(LogParser {
            min_level: min_level.to_string(),
        })
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, LogError> {
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
        let level_order = ["trace", "debug", "info", "warn", "error"];
        let min_index = level_order
            .iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(0);
        let entry_index = level_order.iter().position(|&l| l == level);

        match entry_index {
            Some(idx) => idx >= min_index,
            None => false,
        }
    }

    pub fn filter_by_level(&self, entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
        entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn extract_messages(&self, entries: &[LogEntry]) -> Vec<String> {
        entries
            .iter()
            .map(|entry| entry.message.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"timestamp": "2024-01-01T00:00:00Z", "level": "info", "message": "System started"}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"timestamp": "2024-01-01T00:01:00Z", "level": "error", "message": "Failed to connect", "metadata": {{"code": 500}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"timestamp": "2024-01-01T00:02:00Z", "level": "debug", "message": "Processing data"}}"#
        )
        .unwrap();
        file
    }

    #[test]
    fn test_parse_log_file() {
        let test_file = create_test_log();
        let parser = LogParser::new("info").unwrap();
        let entries = parser.parse_file(test_file.path()).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "info");
        assert_eq!(entries[1].level, "error");
    }

    #[test]
    fn test_filter_by_level() {
        let test_file = create_test_log();
        let parser = LogParser::new("debug").unwrap();
        let entries = parser.parse_file(test_file.path()).unwrap();
        let error_entries = parser.filter_by_level(&entries, "error");

        assert_eq!(error_entries.len(), 1);
        assert_eq!(error_entries[0].message, "Failed to connect");
    }

    #[test]
    fn test_invalid_level() {
        let result = LogParser::new("invalid");
        assert!(matches!(result, Err(LogError::InvalidLevel(_))));
    }
}