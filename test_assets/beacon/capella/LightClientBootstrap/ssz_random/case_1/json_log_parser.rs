
use serde_json::Value;
use std::collections::HashMap;
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
    fn from_str(level: &str) -> Option<Self> {
        match level.to_lowercase().as_str() {
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
    include_fields: Vec<String>,
}

impl LogParser {
    pub fn new(min_level: LogLevel) -> Self {
        LogParser {
            min_level,
            include_fields: Vec::new(),
        }
    }

    pub fn with_fields(mut self, fields: &[&str]) -> Self {
        self.include_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<HashMap<String, Value>>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(parsed) = self.parse_line(&line) {
                logs.push(parsed);
            }
        }

        Ok(logs)
    }

    fn parse_line(&self, line: &str) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
        let json: Value = serde_json::from_str(line)?;
        let mut map = HashMap::new();

        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if self.should_include_field(key) {
                    map.insert(key.clone(), value.clone());
                }
            }
        }

        if let Some(level_str) = map.get("level").and_then(|v| v.as_str()) {
            if let Some(level) = LogLevel::from_str(level_str) {
                if !self.meets_level_requirement(&level) {
                    return Err("Log level below minimum threshold".into());
                }
            }
        }

        Ok(map)
    }

    fn should_include_field(&self, field: &str) -> bool {
        self.include_fields.is_empty() || self.include_fields.contains(&field.to_string())
    }

    fn meets_level_requirement(&self, level: &LogLevel) -> bool {
        match (&self.min_level, level) {
            (LogLevel::Error, _) => level == &LogLevel::Error,
            (LogLevel::Warn, l) => l == &LogLevel::Error || l == &LogLevel::Warn,
            (LogLevel::Info, l) => l != &LogLevel::Debug && l != &LogLevel::Trace,
            (LogLevel::Debug, l) => l != &LogLevel::Trace,
            (LogLevel::Trace, _) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("unknown"), None);
    }

    #[test]
    fn test_level_filtering() {
        let parser = LogParser::new(LogLevel::Info);
        
        assert!(parser.meets_level_requirement(&LogLevel::Error));
        assert!(parser.meets_level_requirement(&LogLevel::Warn));
        assert!(parser.meets_level_requirement(&LogLevel::Info));
        assert!(!parser.meets_level_requirement(&LogLevel::Debug));
        assert!(!parser.meets_level_requirement(&LogLevel::Trace));
    }
}