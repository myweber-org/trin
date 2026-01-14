use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid log level: {0}")]
    InvalidLevel(String),
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: Value,
}

pub struct LogProcessor {
    pub entries: Vec<LogEntry>,
    pub error_count: usize,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            error_count: 0,
        }
    }

    pub fn process_file(&mut self, path: &str) -> Result<(), LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            match self.parse_line(&line) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => {
                    eprintln!("Line {}: {}", line_num + 1, e);
                    self.error_count += 1;
                }
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, LogError> {
        let value: Value = serde_json::from_str(line)?;

        let timestamp = value["timestamp"]
            .as_str()
            .ok_or_else(|| LogError::InvalidLevel("Missing timestamp".to_string()))?
            .to_string();

        let level = value["level"]
            .as_str()
            .ok_or_else(|| LogError::InvalidLevel("Missing level".to_string()))?
            .to_string();

        let message = value["message"]
            .as_str()
            .ok_or_else(|| LogError::InvalidLevel("Missing message".to_string()))?
            .to_string();

        let fields = value["fields"].clone();

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        let error_count = self.filter_by_level("error").len();
        let warn_count = self.filter_by_level("warn").len();
        let info_count = self.filter_by_level("info").len();
        (error_count, warn_count, info_count)
    }
}

impl Default for LogProcessor {
    fn default() -> Self {
        Self::new()
    }
}