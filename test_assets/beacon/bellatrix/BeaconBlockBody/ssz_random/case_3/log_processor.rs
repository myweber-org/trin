
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use chrono::{DateTime, FixedOffset};
use regex::Regex;

pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
}

pub struct LogProcessor {
    timestamp_pattern: Regex,
    level_pattern: Regex,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            timestamp_pattern: Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2}").unwrap(),
            level_pattern: Regex::new(r"\[(ERROR|WARN|INFO|DEBUG|TRACE)\]").unwrap(),
        }
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let timestamp_match = self.timestamp_pattern.find(line)?;
        let level_match = self.level_pattern.captures(line)?;

        let timestamp_str = timestamp_match.as_str();
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str).ok()?;
        
        let level = level_match.get(1)?.as_str().to_string();
        let message = line[timestamp_match.end()..].trim().to_string();

        Some(LogEntry {
            timestamp,
            level,
            message,
        })
    }

    pub fn filter_by_level<'a>(&self, entries: &'a [LogEntry], level: &str) -> Vec<&'a LogEntry> {
        entries.iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn read_log_file(&self, path: &str) -> io::Result<Vec<LogEntry>> {
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
}

pub fn find_errors_in_time_range(
    processor: &LogProcessor,
    path: &str,
    start_time: DateTime<FixedOffset>,
    end_time: DateTime<FixedOffset>,
) -> io::Result<Vec<LogEntry>> {
    let all_entries = processor.read_log_file(path)?;
    
    let filtered: Vec<LogEntry> = all_entries.into_iter()
        .filter(|entry| entry.level == "ERROR")
        .filter(|entry| entry.timestamp >= start_time && entry.timestamp <= end_time)
        .collect();
    
    Ok(filtered)
}