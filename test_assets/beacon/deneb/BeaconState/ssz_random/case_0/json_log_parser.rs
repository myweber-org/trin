use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn filter_logs_by_level(file_path: &str, min_level: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut filtered_logs = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
            if let Some(level) = json_value.get("level").and_then(|v| v.as_str()) {
                if severity_greater_or_equal(level, min_level) {
                    filtered_logs.push(line);
                }
            }
        }
    }

    Ok(filtered_logs)
}

fn severity_greater_or_equal(log_level: &str, min_level: &str) -> bool {
    let levels = ["trace", "debug", "info", "warn", "error", "fatal"];
    let log_idx = levels.iter().position(|&l| l == log_level);
    let min_idx = levels.iter().position(|&l| l == min_level);

    match (log_idx, min_idx) {
        (Some(l), Some(m)) => l >= m,
        _ => false,
    }
}