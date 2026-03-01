
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub struct LogParser {
    pattern: Regex,
}

impl LogParser {
    pub fn new() -> Result<Self, regex::Error> {
        let pattern = Regex::new(r"^(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)$")?;
        Ok(LogParser { pattern })
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<LogEntry>> {
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
        self.pattern.captures(line).map(|caps| LogEntry {
            timestamp: caps[1].to_string(),
            level: caps[2].to_string(),
            message: caps[3].to_string(),
        })
    }

    pub fn filter_by_level(&self, entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
        entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_line() {
        let parser = LogParser::new().unwrap();
        let line = "2023-10-05 14:30:25 [ERROR] Database connection failed";
        let entry = parser.parse_line(line).unwrap();

        assert_eq!(entry.timestamp, "2023-10-05 14:30:25");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
    }

    #[test]
    fn test_parse_invalid_line() {
        let parser = LogParser::new().unwrap();
        let line = "Invalid log format";
        assert!(parser.parse_line(line).is_none());
    }
}