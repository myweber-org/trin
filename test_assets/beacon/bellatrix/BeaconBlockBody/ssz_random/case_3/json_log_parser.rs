use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn parse_json_logs(file_path: &str, level_filter: Option<&str>) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut filtered_logs = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Ok(log_entry) = serde_json::from_str::<Value>(&line) {
            if let Some(filter) = level_filter {
                if let Some(log_level) = log_entry.get("level").and_then(|v| v.as_str()) {
                    if log_level.eq_ignore_ascii_case(filter) {
                        filtered_logs.push(log_entry);
                    }
                }
            } else {
                filtered_logs.push(log_entry);
            }
        }
    }

    Ok(filtered_logs)
}

pub fn count_logs_by_level(logs: &[Value]) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    
    for log in logs {
        if let Some(level) = log.get("level").and_then(|v| v.as_str()) {
            *counts.entry(level.to_string()).or_insert(0) += 1;
        }
    }
    
    counts
}