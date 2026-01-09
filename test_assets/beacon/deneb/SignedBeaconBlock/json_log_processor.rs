use chrono::{DateTime, Utc};
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_json_log_file(
    file_path: &str,
    min_timestamp: Option<DateTime<Utc>>,
    max_timestamp: Option<DateTime<Utc>>,
) -> Result<Vec<LogEntry>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        
        if line.trim().is_empty() {
            continue;
        }

        let json_value: Value = serde_json::from_str(&line)?;
        
        let timestamp_str = json_value["timestamp"]
            .as_str()
            .ok_or("Missing or invalid timestamp field")?;
        
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?
            .with_timezone(&Utc);

        if let Some(min) = min_timestamp {
            if timestamp < min {
                continue;
            }
        }

        if let Some(max) = max_timestamp {
            if timestamp > max {
                continue;
            }
        }

        let entry = LogEntry {
            timestamp,
            level: json_value["level"]
                .as_str()
                .unwrap_or("UNKNOWN")
                .to_string(),
            message: json_value["message"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            metadata: json_value["metadata"].clone(),
        };

        entries.push(entry);
    }

    Ok(entries)
}

pub fn filter_logs_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level.to_uppercase() == level.to_uppercase())
        .collect()
}

pub fn extract_error_messages(entries: &[LogEntry]) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| entry.level.to_uppercase() == "ERROR")
        .map(|entry| entry.message.clone())
        .collect()
}