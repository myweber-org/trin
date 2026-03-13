use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

#[derive(Debug, Error)]
pub enum LogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid log format")]
    InvalidFormat,
}

pub struct LogProcessor {
    entries: Vec<LogEntry>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e),
            }
        }

        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    pub fn count_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn export_to_json(&self, path: &str) -> Result<(), LogError> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &self.entries)?;
        Ok(())
    }
}

impl Default for LogProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "System started"}
{"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "message": "Connection failed", "metadata": {"code": 500}}
{"timestamp": "2024-01-15T10:32:00Z", "level": "WARN", "message": "High memory usage"}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let mut processor = LogProcessor::new();
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.count_entries(), 3);
        assert_eq!(processor.filter_by_level("ERROR").len(), 1);
        assert_eq!(processor.filter_by_level("INFO").len(), 1);
    }
}use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
enum LogError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    InvalidTimestamp(String),
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

struct LogProcessor {
    min_timestamp: Option<DateTime<Utc>>,
    max_timestamp: Option<DateTime<Utc>>,
    level_filter: Option<String>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            min_timestamp: None,
            max_timestamp: None,
            level_filter: None,
        }
    }

    fn with_time_range(mut self, min: Option<DateTime<Utc>>, max: Option<DateTime<Utc>>) -> Self {
        self.min_timestamp = min;
        self.max_timestamp = max;
        self
    }

    fn with_level_filter(mut self, level: Option<String>) -> Self {
        self.level_filter = level;
        self
    }

    fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)?;
            
            if self.filter_entry(&entry) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn filter_entry(&self, entry: &LogEntry) -> bool {
        if let Some(ref level_filter) = self.level_filter {
            if !entry.level.eq_ignore_ascii_case(level_filter) {
                return false;
            }
        }

        match DateTime::parse_from_rfc3339(&entry.timestamp) {
            Ok(dt) => {
                let utc_dt = dt.with_timezone(&Utc);
                
                if let Some(min) = self.min_timestamp {
                    if utc_dt < min {
                        return false;
                    }
                }
                
                if let Some(max) = self.max_timestamp {
                    if utc_dt > max {
                        return false;
                    }
                }
                
                true
            }
            Err(_) => false,
        }
    }
}

fn analyze_logs(entries: &[LogEntry]) -> (usize, Vec<String>, Vec<String>) {
    let total = entries.len();
    let mut services = Vec::new();
    let mut levels = Vec::new();

    for entry in entries {
        if !services.contains(&entry.service) {
            services.push(entry.service.clone());
        }
        if !levels.contains(&entry.level) {
            levels.push(entry.level.clone());
        }
    }

    services.sort();
    levels.sort();
    (total, services, levels)
}

fn main() -> Result<(), LogError> {
    let processor = LogProcessor::new()
        .with_level_filter(Some("ERROR".to_string()))
        .with_time_range(None, Some(Utc::now()));

    let entries = processor.process_file("logs/app.log")?;
    
    let (total, services, levels) = analyze_logs(&entries);
    
    println!("Found {} error entries", total);
    println!("Services: {:?}", services);
    println!("Levels: {:?}", levels);
    
    for entry in entries.iter().take(5) {
        println!("{:?}", entry);
    }
    
    Ok(())
}