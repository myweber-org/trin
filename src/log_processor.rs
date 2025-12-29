use regex::Regex;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PartialEq)]
pub enum LogSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

pub struct LogEntry {
    pub timestamp: u64,
    pub severity: LogSeverity,
    pub message: String,
    pub source: String,
}

pub struct LogProcessor {
    pattern: Regex,
    severity_map: HashMap<String, LogSeverity>,
}

impl LogProcessor {
    pub fn new() -> Self {
        let pattern = Regex::new(r"\[(?P<timestamp>\d+)\] \[(?P<severity>\w+)\] (?P<source>[^:]+): (?P<message>.+)").unwrap();
        
        let mut severity_map = HashMap::new();
        severity_map.insert("INFO".to_string(), LogSeverity::Info);
        severity_map.insert("WARN".to_string(), LogSeverity::Warning);
        severity_map.insert("ERROR".to_string(), LogSeverity::Error);
        severity_map.insert("CRITICAL".to_string(), LogSeverity::Critical);

        LogProcessor { pattern, severity_map }
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        if let Some(captures) = self.pattern.captures(line) {
            let timestamp = captures.name("timestamp")?.as_str().parse().ok()?;
            let severity_str = captures.name("severity")?.as_str();
            let source = captures.name("source")?.as_str().to_string();
            let message = captures.name("message")?.as_str().to_string();

            let severity = self.severity_map.get(severity_str)?;

            Some(LogEntry {
                timestamp,
                severity: severity.clone(),
                message,
                source,
            })
        } else {
            None
        }
    }

    pub fn filter_by_severity(&self, entries: &[LogEntry], min_severity: LogSeverity) -> Vec<&LogEntry> {
        entries.iter()
            .filter(|entry| self.severity_ordinal(&entry.severity) >= self.severity_ordinal(&min_severity))
            .collect()
    }

    fn severity_ordinal(&self, severity: &LogSeverity) -> u8 {
        match severity {
            LogSeverity::Info => 1,
            LogSeverity::Warning => 2,
            LogSeverity::Error => 3,
            LogSeverity::Critical => 4,
        }
    }

    pub fn generate_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_log() {
        let processor = LogProcessor::new();
        let log_line = "[1625097600] [ERROR] auth_service: Invalid credentials provided";
        
        let entry = processor.parse_line(log_line).unwrap();
        
        assert_eq!(entry.timestamp, 1625097600);
        assert_eq!(entry.severity, LogSeverity::Error);
        assert_eq!(entry.source, "auth_service");
        assert_eq!(entry.message, "Invalid credentials provided");
    }

    #[test]
    fn test_filter_severity() {
        let processor = LogProcessor::new();
        let entries = vec![
            LogEntry {
                timestamp: 1625097600,
                severity: LogSeverity::Info,
                message: "Service started".to_string(),
                source: "main".to_string(),
            },
            LogEntry {
                timestamp: 1625097601,
                severity: LogSeverity::Error,
                message: "Database connection failed".to_string(),
                source: "db".to_string(),
            },
        ];

        let filtered = processor.filter_by_severity(&entries, LogSeverity::Warning);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].severity, LogSeverity::Error);
    }
}