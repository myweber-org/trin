use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub component: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum LogParseError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    InvalidTimestamp,
}

impl From<std::io::Error> for LogParseError {
    fn from(err: std::io::Error) -> Self {
        LogParseError::IoError(err)
    }
}

impl From<serde_json::Error> for LogParseError {
    fn from(err: serde_json::Error) -> Self {
        LogParseError::ParseError(err)
    }
}

pub struct LogParser {
    min_level: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            min_level: None,
            start_time: None,
            end_time: None,
        }
    }

    pub fn with_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, LogParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match self.parse_line(&line) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => continue,
                Err(e) => eprintln!("Failed to parse line: {}, error: {:?}", line, e),
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<Option<LogEntry>, LogParseError> {
        let entry: LogEntry = serde_json::from_str(line)?;

        if let Some(min_level) = &self.min_level {
            if !self.is_level_sufficient(&entry.level, min_level) {
                return Ok(None);
            }
        }

        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return Ok(None);
            }
        }

        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return Ok(None);
            }
        }

        Ok(Some(entry))
    }

    fn is_level_sufficient(&self, entry_level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error", "fatal"];
        let entry_idx = levels.iter().position(|&l| l == entry_level.to_lowercase());
        let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());

        match (entry_idx, min_idx) {
            (Some(e), Some(m)) => e >= m,
            _ => false,
        }
    }
}

pub fn analyze_logs(entries: &[LogEntry]) -> (usize, usize, usize, usize) {
    let mut error_count = 0;
    let mut warn_count = 0;
    let mut info_count = 0;
    let mut debug_count = 0;

    for entry in entries {
        match entry.level.to_lowercase().as_str() {
            "error" | "fatal" => error_count += 1,
            "warn" => warn_count += 1,
            "info" => info_count += 1,
            "debug" | "trace" => debug_count += 1,
            _ => {}
        }
    }

    (error_count, warn_count, info_count, debug_count)
}