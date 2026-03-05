use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
    pub metadata: serde_json::Value,
}

pub struct LogParser {
    file_path: String,
    min_level: LogLevel,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
            min_level: LogLevel::INFO,
            start_time: None,
            end_time: None,
        }
    }

    pub fn set_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    pub fn set_time_range(mut self, start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self {
        self.start_time = start;
        self.end_time = end;
        self
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

            let entry: LogEntry = serde_json::from_str(&line)?;

            if !self.is_level_allowed(&entry.level) {
                continue;
            }

            if !self.is_time_in_range(&entry.timestamp) {
                continue;
            }

            entries.push(entry);
        }

        Ok(entries)
    }

    fn is_level_allowed(&self, level: &LogLevel) -> bool {
        let level_value = |lvl: &LogLevel| match lvl {
            LogLevel::ERROR => 4,
            LogLevel::WARN => 3,
            LogLevel::INFO => 2,
            LogLevel::DEBUG => 1,
            LogLevel::TRACE => 0,
        };

        level_value(level) >= level_value(&self.min_level)
    }

    fn is_time_in_range(&self, timestamp: &DateTime<Utc>) -> bool {
        if let Some(start) = &self.start_time {
            if timestamp < start {
                return false;
            }
        }

        if let Some(end) = &self.end_time {
            if timestamp > end {
                return false;
            }
        }

        true
    }
}

pub fn filter_logs_by_component(entries: Vec<LogEntry>, component: &str) -> Vec<LogEntry> {
    entries.into_iter()
        .filter(|entry| entry.component == component)
        .collect()
}

pub fn count_logs_by_level(entries: &[LogEntry]) -> std::collections::HashMap<LogLevel, usize> {
    let mut counts = std::collections::HashMap::new();
    
    for entry in entries {
        *counts.entry(entry.level.clone()).or_insert(0) += 1;
    }
    
    counts
}use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub fn parse_json_log_file(file_path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let json_value: Value = serde_json::from_str(&line)?;
        
        let timestamp = json_value["timestamp"]
            .as_str()
            .unwrap_or("")
            .to_string();
            
        let level = json_value["level"]
            .as_str()
            .unwrap_or("INFO")
            .to_string();
            
        let message = json_value["message"]
            .as_str()
            .unwrap_or("")
            .to_string();

        entries.push(LogEntry {
            timestamp,
            level,
            message,
        });
    }

    Ok(entries)
}

pub fn filter_errors(entries: &[LogEntry]) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level == "ERROR")
        .collect()
}