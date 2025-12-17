use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error at line {0}: {1}")]
    JsonParse(usize, serde_json::Error),
    #[error("Missing required field '{0}' at line {1}")]
    MissingField(&'static str, usize),
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_json_log_file(path: &str) -> Result<Vec<LogEntry>, LogParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let line_index = line_num + 1;

        let json_value: Value = serde_json::from_str(&line)
            .map_err(|e| LogParseError::JsonParse(line_index, e))?;

        let entry = parse_log_entry(json_value, line_index)?;
        entries.push(entry);
    }

    Ok(entries)
}

fn parse_log_entry(value: Value, line_num: usize) -> Result<LogEntry, LogParseError> {
    let obj = value.as_object()
        .ok_or_else(|| LogParseError::MissingField("object", line_num))?;

    let timestamp = obj.get("timestamp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogParseError::MissingField("timestamp", line_num))?
        .to_string();

    let level = obj.get("level")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogParseError::MissingField("level", line_num))?
        .to_string();

    let message = obj.get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogParseError::MissingField("message", line_num))?
        .to_string();

    let metadata = obj.get("metadata")
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    Ok(LogEntry {
        timestamp,
        level,
        message,
        metadata,
    })
}

pub fn filter_logs_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|entry| entry.level.eq_ignore_ascii_case(level))
        .collect()
}

pub fn extract_timestamps(entries: &[LogEntry]) -> Vec<&str> {
    entries.iter()
        .map(|entry| entry.timestamp.as_str())
        .collect()
}