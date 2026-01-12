use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;
use chrono::NaiveDateTime;

pub struct LogEntry {
    pub timestamp: NaiveDateTime,
    pub level: String,
    pub message: String,
}

pub struct LogParser {
    pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        let pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)").unwrap();
        LogParser { pattern }
    }

    pub fn parse_file(&self, path: &str) -> io::Result<Vec<LogEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let captures = self.pattern.captures(line)?;
        
        let timestamp_str = captures.get(1)?.as_str();
        let level = captures.get(2)?.as_str().to_string();
        let message = captures.get(3)?.as_str().to_string();

        let timestamp = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S").ok()?;

        Some(LogEntry {
            timestamp,
            level,
            message,
        })
    }

    pub fn filter_by_level(&self, entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
        entries.iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn find_errors(&self, entries: &[LogEntry]) -> Vec<&LogEntry> {
        self.filter_by_level(entries, "ERROR")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_line() {
        let parser = LogParser::new();
        let line = "2024-01-15 14:30:45 [ERROR] Database connection failed";
        
        let entry = parser.parse_line(line).unwrap();
        
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
    }

    #[test]
    fn test_parse_invalid_line() {
        let parser = LogParser::new();
        let line = "Invalid log format";
        
        assert!(parser.parse_line(line).is_none());
    }
}