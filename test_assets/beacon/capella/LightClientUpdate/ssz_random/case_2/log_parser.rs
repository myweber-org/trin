
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

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
        let pattern = Regex::new(r"\[(?P<timestamp>[^\]]+)\] (?P<level>\w+): (?P<message>.+)")?;
        Ok(LogParser { pattern })
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        self.pattern.captures(line).map(|caps| LogEntry {
            timestamp: caps["timestamp"].to_string(),
            level: caps["level"].to_string(),
            message: caps["message"].to_string(),
        })
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<LogEntry>> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        
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
}

pub fn filter_errors(entries: &[LogEntry]) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|entry| entry.level == "ERROR")
        .collect()
}