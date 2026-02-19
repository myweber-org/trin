use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+): (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp = captures[1].to_string();
                let level = captures[2].to_string();
                let message = captures[3].to_string();

                let entry = LogEntry {
                    timestamp,
                    level: level.clone(),
                    message,
                };

                *self.level_counts.entry(level).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn get_summary(&self) -> HashMap<String, usize> {
        self.level_counts.clone()
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = LogAnalyzer::new();
        assert_eq!(analyzer.total_entries(), 0);
    }

    #[test]
    fn test_filter_function() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.entries.push(LogEntry {
            timestamp: "2024-01-01 10:00:00".to_string(),
            level: "ERROR".to_string(),
            message: "Test error message".to_string(),
        });
        
        let filtered = analyzer.filter_by_level("ERROR");
        assert_eq!(filtered.len(), 1);
    }
}use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use chrono::{DateTime, FixedOffset};

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub component: String,
    pub message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let timestamp_str = parts[0].trim();
        let level = parts[1].trim().to_string();
        let component = parts[2].trim().to_string();
        let message = parts[3].trim().to_string();

        match DateTime::parse_from_rfc3339(timestamp_str) {
            Ok(timestamp) => Some(LogEntry {
                timestamp,
                level,
                component,
                message,
            }),
            Err(_) => None,
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .cloned()
            .collect()
    }

    pub fn filter_by_component(&self, component: &str) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.component.contains(component))
            .cloned()
            .collect()
    }

    pub fn get_entries_in_time_range(
        &self,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
    ) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .cloned()
            .collect()
    }

    pub fn count_by_level(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn get_latest_entries(&self, count: usize) -> Vec<LogEntry> {
        let mut sorted_entries = self.entries.clone();
        sorted_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted_entries.into_iter().take(count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_parse_log_line() {
        let analyzer = LogAnalyzer::new();
        let line = "2023-10-05T14:30:00+00:00 | INFO | network | Connection established";
        let entry = analyzer.parse_log_line(line).unwrap();

        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.component, "network");
        assert_eq!(entry.message, "Connection established");
    }

    #[test]
    fn test_filter_by_level() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.entries.push(LogEntry {
            timestamp: FixedOffset::east(0).ymd(2023, 10, 5).and_hms(14, 30, 0),
            level: "ERROR".to_string(),
            component: "database".to_string(),
            message: "Connection failed".to_string(),
        });
        analyzer.entries.push(LogEntry {
            timestamp: FixedOffset::east(0).ymd(2023, 10, 5).and_hms(14, 31, 0),
            level: "INFO".to_string(),
            component: "network".to_string(),
            message: "Request processed".to_string(),
        });

        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].component, "database");
    }
}