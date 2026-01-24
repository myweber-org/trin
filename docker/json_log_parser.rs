use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level == level)
            .collect();

        Ok(filtered)
    }

    pub fn count_entries(&self) -> Result<usize, Box<dyn Error>> {
        let entries = self.parse()?;
        Ok(entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_log_entries() {
        let json_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","service":"api","message":"Request received","metadata":{"user_id":123}}
{"timestamp":"2023-10-01T12:01:00Z","level":"ERROR","service":"db","message":"Connection failed","metadata":{"retry_count":3}}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, LogLevel::INFO);
        assert_eq!(entries[1].level, LogLevel::ERROR);
    }

    #[test]
    fn test_filter_error_logs() {
        let json_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","service":"api","message":"Request received"}
{"timestamp":"2023-10-01T12:01:00Z","level":"ERROR","service":"db","message":"Connection failed"}
{"timestamp":"2023-10-01T12:02:00Z","level":"WARN","service":"cache","message":"High memory usage"}
{"timestamp":"2023-10-01T12:03:00Z","level":"ERROR","service":"api","message":"Timeout occurred"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level(LogLevel::ERROR).unwrap();

        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|entry| entry.level == LogLevel::ERROR));
    }
}use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
            }
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level == level)
            .collect();
        Ok(filtered)
    }

    pub fn count_by_service(&self) -> Result<std::collections::HashMap<String, usize>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let mut counts = std::collections::HashMap::new();

        for entry in entries {
            *counts.entry(entry.service).or_insert(0) += 1;
        }

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_log_entries() {
        let json_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"Error","service":"api","message":"Connection failed","metadata":{"error_code":500}}
{"timestamp":"2024-01-15T10:31:00Z","level":"Info","service":"auth","message":"User login successful","metadata":null}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, LogLevel::Error);
        assert_eq!(entries[1].service, "auth");
    }

    #[test]
    fn test_filter_by_level() {
        let json_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"Error","service":"api","message":"Error 1","metadata":null}
{"timestamp":"2024-01-15T10:31:00Z","level":"Info","service":"auth","message":"Info 1","metadata":null}
{"timestamp":"2024-01-15T10:32:00Z","level":"Error","service":"db","message":"Error 2","metadata":null}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level(LogLevel::Error).unwrap();

        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e.level == LogLevel::Error));
    }
}