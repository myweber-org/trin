use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    ValidationError(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<serde_json::Error> for LogError {
    fn from(err: serde_json::Error) -> Self {
        LogError::ParseError(err)
    }
}

impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogError::IoError(e) => write!(f, "IO error: {}", e),
            LogError::ParseError(e) => write!(f, "Parse error: {}", e),
            LogError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl Error for LogError {}

pub struct LogProcessor {
    entries: Vec<LogEntry>,
    stats: ProcessingStats,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_lines: usize,
    pub valid_entries: usize,
    pub errors: usize,
    pub level_counts: HashMap<String, usize>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            stats: ProcessingStats::default(),
        }
    }

    pub fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            self.stats.total_lines += 1;
            let line = line_result?;

            match self.parse_line(&line) {
                Ok(entry) => {
                    self.entries.push(entry.clone());
                    self.stats.valid_entries += 1;
                    *self.stats.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
                }
                Err(e) => {
                    self.stats.errors += 1;
                    eprintln!("Failed to parse line {}: {}", self.stats.total_lines, e);
                }
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, LogError> {
        let parsed: Value = serde_json::from_str(line)?;
        
        let timestamp = parsed["timestamp"]
            .as_str()
            .ok_or_else(|| LogError::ValidationError("Missing timestamp".to_string()))?
            .to_string();

        let level = parsed["level"]
            .as_str()
            .ok_or_else(|| LogError::ValidationError("Missing level".to_string()))?
            .to_string();

        let service = parsed["service"]
            .as_str()
            .ok_or_else(|| LogError::ValidationError("Missing service".to_string()))?
            .to_string();

        let message = parsed["message"]
            .as_str()
            .ok_or_else(|| LogError::ValidationError("Missing message".to_string()))?
            .to_string();

        let metadata = if let Some(obj) = parsed["metadata"].as_object() {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else {
            HashMap::new()
        };

        Ok(LogEntry {
            timestamp,
            level,
            service,
            message,
            metadata,
        })
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    pub fn get_stats(&self) -> &ProcessingStats {
        &self.stats
    }

    pub fn export_to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_log_parsing() {
        let mut processor = LogProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","service":"api-gateway","message":"Connection timeout","metadata":{"duration":5000,"endpoint":"/api/users"}}"#;
        writeln!(temp_file, "{}", log_line).unwrap();
        
        processor.process_file(temp_file.path()).unwrap();
        let stats = processor.get_stats();
        
        assert_eq!(stats.total_lines, 1);
        assert_eq!(stats.valid_entries, 1);
        assert_eq!(stats.errors, 0);
        assert_eq!(stats.level_counts.get("ERROR"), Some(&1));
    }

    #[test]
    fn test_invalid_json() {
        let mut processor = LogProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "invalid json").unwrap();
        
        processor.process_file(temp_file.path()).unwrap();
        let stats = processor.get_stats();
        
        assert_eq!(stats.total_lines, 1);
        assert_eq!(stats.valid_entries, 0);
        assert_eq!(stats.errors, 1);
    }

    #[test]
    fn test_filtering() {
        let mut processor = LogProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let logs = vec![
            r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","service":"api","message":"Error 1"}"#,
            r#"{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","service":"api","message":"Info 1"}"#,
            r#"{"timestamp":"2024-01-15T10:32:00Z","level":"ERROR","service":"db","message":"Error 2"}"#,
        ];
        
        for log in logs {
            writeln!(temp_file, "{}", log).unwrap();
        }
        
        processor.process_file(temp_file.path()).unwrap();
        
        let error_logs = processor.filter_by_level("ERROR");
        assert_eq!(error_logs.len(), 2);
        
        let api_logs = processor.filter_by_service("api");
        assert_eq!(api_logs.len(), 2);
    }
}