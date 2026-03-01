
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogProcessor {
    pattern: Regex,
}

impl LogProcessor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let pattern = Regex::new(r"\[(?P<timestamp>[^\]]+)\] \[(?P<level>[^\]]+)\] (?P<message>.+)")?;
        Ok(LogProcessor { pattern })
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        self.pattern.captures(line).map(|caps| LogEntry {
            timestamp: caps["timestamp"].to_string(),
            level: caps["level"].to_string(),
            message: caps["message"].to_string(),
        })
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Some(entry) = self.parse_line(&line) {
                    entries.push(entry);
                }
            }
        }
        Ok(entries)
    }

    pub fn count_by_level(&self, entries: &[LogEntry]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn filter_by_level(&self, entries: Vec<LogEntry>, level: &str) -> Vec<LogEntry> {
        entries.into_iter()
            .filter(|entry| entry.level == level)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_line() {
        let processor = LogProcessor::new().unwrap();
        let line = "[2023-10-05T14:30:00Z] [ERROR] Database connection failed";
        let entry = processor.parse_line(line).unwrap();
        
        assert_eq!(entry.timestamp, "2023-10-05T14:30:00Z");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
    }

    #[test]
    fn test_parse_invalid_line() {
        let processor = LogProcessor::new().unwrap();
        let line = "Invalid log format without brackets";
        assert!(processor.parse_line(line).is_none());
    }

    #[test]
    fn test_count_levels() {
        let processor = LogProcessor::new().unwrap();
        let entries = vec![
            LogEntry { timestamp: "2023-10-05T14:30:00Z".to_string(), level: "ERROR".to_string(), message: "Msg1".to_string() },
            LogEntry { timestamp: "2023-10-05T14:31:00Z".to_string(), level: "INFO".to_string(), message: "Msg2".to_string() },
            LogEntry { timestamp: "2023-10-05T14:32:00Z".to_string(), level: "ERROR".to_string(), message: "Msg3".to_string() },
        ];
        
        let counts = processor.count_by_level(&entries);
        assert_eq!(counts.get("ERROR"), Some(&2));
        assert_eq!(counts.get("INFO"), Some(&1));
    }
}