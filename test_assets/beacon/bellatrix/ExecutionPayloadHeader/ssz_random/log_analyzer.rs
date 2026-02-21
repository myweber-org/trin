use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub component: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogAnalyzer {
    pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        let pattern = Regex::new(
            r"(?x)
            ^(?P<timestamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2})\s+
            \[(?P<level>\w+)\]\s+
            (?P<component>\w+):\s+
            (?P<message>.*?)\s*
            (?:\{(?P<metadata>.*)\})?$"
        ).unwrap();
        
        LogAnalyzer { pattern }
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let caps = self.pattern.captures(line)?;
        
        let timestamp = DateTime::parse_from_rfc3339(caps.name("timestamp")?.as_str()).ok()?;
        let level = caps.name("level")?.as_str().to_string();
        let component = caps.name("component")?.as_str().to_string();
        let message = caps.name("message")?.as_str().to_string();
        
        let mut metadata = HashMap::new();
        if let Some(meta_str) = caps.name("metadata") {
            for pair in meta_str.as_str().split(',') {
                let mut kv = pair.split('=');
                if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                    metadata.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
        
        Some(LogEntry {
            timestamp,
            level,
            component,
            message,
            metadata,
        })
    }

    pub fn analyze_file(&self, path: &str) -> Result<Vec<LogEntry>, std::io::Error> {
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

    pub fn filter_by_level(&self, entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
        entries.iter()
            .filter(|e| e.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn count_by_component(&self, entries: &[LogEntry]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in entries {
            *counts.entry(entry.component.clone()).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_log() {
        let analyzer = LogAnalyzer::new();
        let line = "2024-01-15T14:30:45+00:00 [INFO] network: Connection established {user_id=123, ip=192.168.1.1}";
        
        let entry = analyzer.parse_line(line).unwrap();
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.component, "network");
        assert_eq!(entry.message, "Connection established");
        assert_eq!(entry.metadata.get("user_id").unwrap(), "123");
    }

    #[test]
    fn test_filter_logs() {
        let analyzer = LogAnalyzer::new();
        let entries = vec![
            LogEntry {
                timestamp: DateTime::parse_from_rfc3339("2024-01-15T10:00:00+00:00").unwrap(),
                level: "ERROR".to_string(),
                component: "database".to_string(),
                message: "Connection failed".to_string(),
                metadata: HashMap::new(),
            },
            LogEntry {
                timestamp: DateTime::parse_from_rfc3339("2024-01-15T10:01:00+00:00").unwrap(),
                level: "INFO".to_string(),
                component: "database".to_string(),
                message: "Connection restored".to_string(),
                metadata: HashMap::new(),
            },
        ];
        
        let errors = analyzer.filter_by_level(&entries, "ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Connection failed");
    }
}