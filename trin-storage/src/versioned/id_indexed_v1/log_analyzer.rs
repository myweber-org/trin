use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    module: String,
    message: String,
    metadata: HashMap<String, String>,
}

impl LogEntry {
    pub fn new(timestamp: DateTime<FixedOffset>, level: &str, module: &str, message: &str) -> Self {
        LogEntry {
            timestamp,
            level: level.to_string(),
            module: module.to_string(),
            message: message.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn matches_filter(&self, level_filter: Option<&str>, module_filter: Option<&str>) -> bool {
        if let Some(level) = level_filter {
            if !self.level.eq_ignore_ascii_case(level) {
                return false;
            }
        }

        if let Some(module) = module_filter {
            if !self.module.contains(module) {
                return false;
            }
        }

        true
    }
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

    pub fn load_from_file(&mut self, path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
                count += 1;
            }
        }

        Ok(count)
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() < 4 {
            return None;
        }

        let timestamp_str = format!("{} {}", parts[0], parts[1]);
        let level = parts[2];
        let module_message = parts[3];
        
        let module_end = module_message.find(':')?;
        let module = &module_message[..module_end];
        let message = &module_message[module_end + 2..];

        let timestamp = DateTime::parse_from_str(&timestamp_str, "%Y-%m-%d %H:%M:%S %z")
            .ok()?;

        Some(LogEntry::new(timestamp, level, module, message))
    }

    pub fn filter_entries(&self, level: Option<&str>, module: Option<&str>) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.matches_filter(level, module))
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }

        counts
    }

    pub fn get_entries_in_time_range(
        &self,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
    ) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_entry_creation() {
        let timestamp = FixedOffset::east(3600)
            .ymd(2023, 10, 15)
            .and_hms(14, 30, 0);
        
        let entry = LogEntry::new(timestamp, "INFO", "network", "Connection established");
        
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.module, "network");
        assert_eq!(entry.message, "Connection established");
    }

    #[test]
    fn test_filter_matching() {
        let timestamp = FixedOffset::east(3600)
            .ymd(2023, 10, 15)
            .and_hms(14, 30, 0);
        
        let entry = LogEntry::new(timestamp, "ERROR", "database", "Connection timeout");
        
        assert!(entry.matches_filter(Some("ERROR"), Some("data")));
        assert!(!entry.matches_filter(Some("INFO"), Some("data")));
        assert!(entry.matches_filter(None, Some("database")));
    }
}