use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub module: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogAnalyzer {
    pub entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = Self::parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(LogAnalyzer { entries })
    }

    fn parse_line(line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() != 5 {
            return None;
        }

        let timestamp_str = parts[0].trim();
        let level = parts[1].trim().to_string();
        let module = parts[2].trim().to_string();
        let message = parts[3].trim().to_string();
        let metadata_str = parts[4].trim();

        let timestamp = match DateTime::parse_from_rfc3339(timestamp_str) {
            Ok(dt) => dt,
            Err(_) => return None,
        };

        let metadata = Self::parse_metadata(metadata_str);

        Some(LogEntry {
            timestamp,
            level,
            module,
            message,
            metadata,
        })
    }

    fn parse_metadata(metadata_str: &str) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        for pair in metadata_str.split(',') {
            let kv: Vec<&str> = pair.splitn(2, '=').collect();
            if kv.len() == 2 {
                metadata.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
            }
        }
        metadata
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn filter_by_module(&self, module: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.module.contains(module))
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn find_errors(&self) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.level.to_lowercase() == "error"
                    || entry.level.to_lowercase() == "critical"
                    || entry.level.to_lowercase() == "fatal"
            })
            .collect()
    }

    pub fn get_time_range(&self) -> Option<(DateTime<FixedOffset>, DateTime<FixedOffset>)> {
        if self.entries.is_empty() {
            return None;
        }

        let mut min_time = &self.entries[0].timestamp;
        let mut max_time = &self.entries[0].timestamp;

        for entry in &self.entries {
            if entry.timestamp < *min_time {
                min_time = &entry.timestamp;
            }
            if entry.timestamp > *max_time {
                max_time = &entry.timestamp;
            }
        }

        Some((*min_time, *max_time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_line() {
        let line = "2023-10-05T14:30:00+00:00 | INFO | network | Connection established | user_id=123,ip=192.168.1.1";
        let entry = LogAnalyzer::parse_line(line).unwrap();

        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.module, "network");
        assert_eq!(entry.message, "Connection established");
        assert_eq!(entry.metadata.get("user_id"), Some(&"123".to_string()));
        assert_eq!(entry.metadata.get("ip"), Some(&"192.168.1.1".to_string()));
    }

    #[test]
    fn test_parse_invalid_line() {
        let line = "Invalid log line without proper format";
        assert!(LogAnalyzer::parse_line(line).is_none());
    }
}