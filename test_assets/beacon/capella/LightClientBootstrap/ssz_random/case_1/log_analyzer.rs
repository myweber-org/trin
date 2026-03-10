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

    pub fn parse_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)")?;

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

    pub fn get_level_summary(&self) -> &HashMap<String, usize> {
        &self.level_counts
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn contains_error(&self) -> bool {
        self.level_counts.contains_key("ERROR")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "2024-01-15 10:30:00 [INFO] Application started"
        ).unwrap();
        writeln!(
            temp_file,
            "2024-01-15 10:31:00 [ERROR] Failed to connect to database"
        ).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.total_entries(), 2);
        assert!(analyzer.contains_error());
        assert_eq!(analyzer.filter_by_level("ERROR").len(), 1);
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    component: String,
    message: String,
    metadata: HashMap<String, String>,
}

impl LogEntry {
    pub fn new(timestamp: DateTime<FixedOffset>, level: &str, component: &str, message: &str) -> Self {
        LogEntry {
            timestamp,
            level: level.to_string(),
            component: component.to_string(),
            message: message.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn is_error(&self) -> bool {
        self.level.to_uppercase() == "ERROR"
    }

    pub fn matches_component(&self, component_filter: &str) -> bool {
        self.component == component_filter
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

    pub fn load_from_file(&mut self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(filepath)?;
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
        let level = parts[1].trim();
        let component = parts[2].trim();
        let message = parts[3].trim();

        if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp_str) {
            Some(LogEntry::new(timestamp, level, component, message))
        } else {
            None
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_uppercase() == level.to_uppercase())
            .collect()
    }

    pub fn filter_by_component(&self, component: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.component == component)
            .collect()
    }

    pub fn get_error_count(&self) -> usize {
        self.entries.iter().filter(|entry| entry.is_error()).count()
    }

    pub fn get_unique_components(&self) -> Vec<String> {
        let mut components: Vec<String> = self.entries
            .iter()
            .map(|entry| entry.component.clone())
            .collect();
        
        components.sort();
        components.dedup();
        components
    }

    pub fn get_earliest_timestamp(&self) -> Option<DateTime<FixedOffset>> {
        self.entries.iter().map(|entry| entry.timestamp).min()
    }

    pub fn get_latest_timestamp(&self) -> Option<DateTime<FixedOffset>> {
        self.entries.iter().map(|entry| entry.timestamp).max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_entry_creation() {
        let timestamp = FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2023, 10, 5, 14, 30, 0)
            .unwrap();
        
        let entry = LogEntry::new(timestamp, "INFO", "auth", "User login successful");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.component, "auth");
        assert!(entry.message.contains("login"));
        assert!(!entry.is_error());
    }

    #[test]
    fn test_error_detection() {
        let timestamp = FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2023, 10, 5, 14, 30, 0)
            .unwrap();
        
        let error_entry = LogEntry::new(timestamp, "ERROR", "database", "Connection failed");
        let info_entry = LogEntry::new(timestamp, "INFO", "database", "Connection established");
        
        assert!(error_entry.is_error());
        assert!(!info_entry.is_error());
    }

    #[test]
    fn test_component_filter() {
        let timestamp = FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2023, 10, 5, 14, 30, 0)
            .unwrap();
        
        let entry = LogEntry::new(timestamp, "WARN", "network", "High latency detected");
        assert!(entry.matches_component("network"));
        assert!(!entry.matches_component("database"));
    }
}