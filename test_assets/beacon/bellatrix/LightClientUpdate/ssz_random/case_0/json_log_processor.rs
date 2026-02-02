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
}