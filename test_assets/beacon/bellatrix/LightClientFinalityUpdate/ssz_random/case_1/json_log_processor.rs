use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogProcessor {
    min_level: LogLevel,
    service_filter: Option<String>,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel) -> Self {
        LogProcessor {
            min_level,
            service_filter: None,
        }
    }

    pub fn with_service_filter(mut self, service: &str) -> Self {
        self.service_filter = Some(service.to_string());
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.should_include(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        if !self.is_level_sufficient(&entry.level) {
            return false;
        }

        if let Some(ref service) = self.service_filter {
            if entry.service != *service {
                return false;
            }
        }

        true
    }

    fn is_level_sufficient(&self, level: &LogLevel) -> bool {
        match (&self.min_level, level) {
            (LogLevel::Error, _) => matches!(level, LogLevel::Error),
            (LogLevel::Warn, _) => matches!(level, LogLevel::Error | LogLevel::Warn),
            (LogLevel::Info, _) => matches!(level, LogLevel::Error | LogLevel::Warn | LogLevel::Info),
            (LogLevel::Debug, _) => matches!(level, LogLevel::Error | LogLevel::Warn | LogLevel::Info | LogLevel::Debug),
            (LogLevel::Trace, _) => true,
        }
    }

    pub fn count_by_level(&self, entries: &[LogEntry]) -> HashMap<LogLevel, usize> {
        let mut counts = HashMap::new();
        for entry in entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"Error","service":"api","message":"Connection failed","metadata":{"ip":"192.168.1.1"}}
{"timestamp":"2024-01-15T10:31:00Z","level":"Info","service":"auth","message":"User login","metadata":{"user_id":"123"}}
{"timestamp":"2024-01-15T10:32:00Z","level":"Warn","service":"api","message":"High latency","metadata":{"duration":"1500ms"}}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let processor = LogProcessor::new(LogLevel::Warn)
            .with_service_filter("api");

        let entries = processor.process_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 2);

        let counts = processor.count_by_level(&entries);
        assert_eq!(counts.get(&LogLevel::Error), Some(&1));
        assert_eq!(counts.get(&LogLevel::Warn), Some(&1));
        assert_eq!(counts.get(&LogLevel::Info), None);
    }
}