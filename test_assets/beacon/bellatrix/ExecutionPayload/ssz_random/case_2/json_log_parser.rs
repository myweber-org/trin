use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warn" => Some(LogLevel::Warn),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            "trace" => Some(LogLevel::Trace),
            _ => None,
        }
    }
}

pub struct LogParser {
    min_level: LogLevel,
}

impl LogParser {
    pub fn new(min_level: LogLevel) -> Self {
        LogParser { min_level }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut filtered_logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if self.should_include(&json_value) {
                    filtered_logs.push(json_value);
                }
            }
        }

        Ok(filtered_logs)
    }

    fn should_include(&self, log_entry: &Value) -> bool {
        let level_priority = |level: &LogLevel| match level {
            LogLevel::Error => 4,
            LogLevel::Warn => 3,
            LogLevel::Info => 2,
            LogLevel::Debug => 1,
            LogLevel::Trace => 0,
        };

        if let Some(level_str) = log_entry.get("level").and_then(|v| v.as_str()) {
            if let Some(log_level) = LogLevel::from_string(level_str) {
                return level_priority(&log_level) >= level_priority(&self.min_level);
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_log_level_filtering() {
        let parser = LogParser::new(LogLevel::Info);
        
        let test_logs = vec![
            json!({"level": "error", "message": "Critical failure"}),
            json!({"level": "warn", "message": "Potential issue"}),
            json!({"level": "info", "message": "System started"}),
            json!({"level": "debug", "message": "Debug data"}),
        ];

        let included: Vec<_> = test_logs.iter()
            .filter(|log| parser.should_include(log))
            .collect();

        assert_eq!(included.len(), 3);
        assert!(included.contains(&&json!({"level": "error", "message": "Critical failure"})));
        assert!(included.contains(&&json!({"level": "warn", "message": "Potential issue"})));
        assert!(included.contains(&&json!({"level": "info", "message": "System started"})));
    }
}