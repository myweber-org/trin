use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;

#[derive(Debug, PartialEq, Eq)]
enum LogSeverity {
    Info,
    Warning,
    Error,
    Debug,
    Unknown,
}

impl From<&str> for LogSeverity {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "info" => LogSeverity::Info,
            "warning" => LogSeverity::Warning,
            "error" => LogSeverity::Error,
            "debug" => LogSeverity::Debug,
            _ => LogSeverity::Unknown,
        }
    }
}

pub struct LogParser {
    file_path: String,
    severity_filter: Option<LogSeverity>,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
            severity_filter: None,
        }
    }

    pub fn set_severity_filter(&mut self, severity: LogSeverity) {
        self.severity_filter = Some(severity);
    }

    pub fn parse(&self) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                let mut log_entry = HashMap::new();

                if let Some(timestamp) = json_value.get("timestamp").and_then(|v| v.as_str()) {
                    log_entry.insert("timestamp".to_string(), timestamp.to_string());
                }

                if let Some(severity) = json_value.get("severity").and_then(|v| v.as_str()) {
                    log_entry.insert("severity".to_string(), severity.to_string());
                }

                if let Some(message) = json_value.get("message").and_then(|v| v.as_str()) {
                    log_entry.insert("message".to_string(), message.to_string());
                }

                if let Some(component) = json_value.get("component").and_then(|v| v.as_str()) {
                    log_entry.insert("component".to_string(), component.to_string());
                }

                if let Some(filter) = &self.severity_filter {
                    if let Some(entry_severity) = log_entry.get("severity") {
                        let entry_sev: LogSeverity = entry_severity.as_str().into();
                        if &entry_sev != filter {
                            continue;
                        }
                    }
                }

                logs.push(log_entry);
            }
        }

        Ok(logs)
    }

    pub fn count_by_severity(&self) -> Result<HashMap<String, usize>, Box<dyn std::error::Error>> {
        let logs = self.parse()?;
        let mut counts = HashMap::new();

        for log in logs {
            if let Some(severity) = log.get("severity") {
                *counts.entry(severity.clone()).or_insert(0) += 1;
            }
        }

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let log_lines = vec![
            r#"{"timestamp": "2023-10-01T12:00:00Z", "severity": "INFO", "message": "System started", "component": "boot"}"#,
            r#"{"timestamp": "2023-10-01T12:01:00Z", "severity": "WARNING", "message": "High memory usage", "component": "monitor"}"#,
            r#"{"timestamp": "2023-10-01T12:02:00Z", "severity": "ERROR", "message": "Disk write failed", "component": "storage"}"#,
            r#"{"timestamp": "2023-10-01T12:03:00Z", "severity": "INFO", "message": "User login", "component": "auth"}"#,
            r#"{"timestamp": "2023-10-01T12:04:00Z", "severity": "DEBUG", "message": "Cache hit", "component": "cache"}"#,
        ];

        for line in log_lines {
            writeln!(file, "{}", line).unwrap();
        }

        file
    }

    #[test]
    fn test_parse_all_logs() {
        let file = create_test_log_file();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let logs = parser.parse().unwrap();
        assert_eq!(logs.len(), 5);
    }

    #[test]
    fn test_filter_by_severity() {
        let file = create_test_log_file();
        let mut parser = LogParser::new(file.path().to_str().unwrap());
        parser.set_severity_filter(LogSeverity::Info);
        let logs = parser.parse().unwrap();
        assert_eq!(logs.len(), 2);
        for log in logs {
            assert_eq!(log.get("severity").unwrap(), "INFO");
        }
    }

    #[test]
    fn test_count_by_severity() {
        let file = create_test_log_file();
        let parser = LogParser::new(file.path().to_str().unwrap());
        let counts = parser.count_by_severity().unwrap();
        assert_eq!(counts.get("INFO"), Some(&2));
        assert_eq!(counts.get("WARNING"), Some(&1));
        assert_eq!(counts.get("ERROR"), Some(&1));
        assert_eq!(counts.get("DEBUG"), Some(&1));
    }
}