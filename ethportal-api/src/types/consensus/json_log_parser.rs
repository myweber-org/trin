use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidTimestamp,
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
    min_level: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            min_level: None,
            start_time: None,
            end_time: None,
        }
    }

    pub fn with_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)?;
            
            if self.filter_entry(&entry) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn filter_entry(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let entry_level = entry.level.to_lowercase();
            let level_order = ["trace", "debug", "info", "warn", "error"];
            
            let entry_idx = level_order.iter().position(|&l| l == entry_level);
            let min_idx = level_order.iter().position(|&l| l == min_level.as_str());
            
            if let (Some(ei), Some(mi)) = (entry_idx, min_idx) {
                if ei < mi {
                    return false;
                }
            }
        }

        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }

        true
    }

    pub fn count_by_level(&self, entries: &[LogEntry]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for entry in entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","fields":{{"service":"api"}}}}"#).unwrap();
        writeln!(file, r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Database connection failed","fields":{{"service":"db","attempt":"3"}}}}"#).unwrap();
        writeln!(file, r#"{{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","message":"High memory usage","fields":{{"service":"api","usage":"85%"}}}}"#).unwrap();
        file
    }

    #[test]
    fn test_parse_log_file() {
        let file = create_test_log();
        let parser = LogParser::new();
        let entries = parser.parse_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].message, "Database connection failed");
    }

    #[test]
    fn test_filter_by_level() {
        let file = create_test_log();
        let parser = LogParser::new().with_min_level("WARN");
        let entries = parser.parse_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.level == "ERROR" || e.level == "WARN"));
    }

    #[test]
    fn test_count_by_level() {
        let file = create_test_log();
        let parser = LogParser::new();
        let entries = parser.parse_file(file.path().to_str().unwrap()).unwrap();
        let counts = parser.count_by_level(&entries);
        
        assert_eq!(counts.get("INFO"), Some(&1));
        assert_eq!(counts.get("ERROR"), Some(&1));
        assert_eq!(counts.get("WARN"), Some(&1));
    }
}