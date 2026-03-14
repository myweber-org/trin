use std::collections::HashMap;
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

    pub fn matches_filter(&self, level_filter: Option<&str>, component_filter: Option<&str>) -> bool {
        if let Some(level) = level_filter {
            if !self.level.eq_ignore_ascii_case(level) {
                return false;
            }
        }
        
        if let Some(component) = component_filter {
            if !self.component.contains(component) {
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

    pub fn load_from_file(&mut self, path: &str) -> std::io::Result<usize> {
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
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        match DateTime::parse_from_rfc3339(parts[0].trim()) {
            Ok(timestamp) => {
                let mut entry = LogEntry::new(
                    timestamp,
                    parts[1].trim(),
                    parts[2].trim(),
                    parts[3].trim(),
                );

                if let Some(metadata_start) = parts[3].find('{') {
                    if let Some(metadata_end) = parts[3].find('}') {
                        let metadata_str = &parts[3][metadata_start + 1..metadata_end];
                        for pair in metadata_str.split(',') {
                            let kv: Vec<&str> = pair.split('=').collect();
                            if kv.len() == 2 {
                                entry.add_metadata(kv[0].trim(), kv[1].trim());
                            }
                        }
                    }
                }

                Some(entry)
            }
            Err(_) => None,
        }
    }

    pub fn filter_logs(&self, level: Option<&str>, component: Option<&str>) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.matches_filter(level, component))
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn get_errors(&self) -> Vec<&LogEntry> {
        self.filter_logs(Some("ERROR"), None)
    }

    pub fn get_warnings(&self) -> Vec<&LogEntry> {
        self.filter_logs(Some("WARN"), None)
    }
}