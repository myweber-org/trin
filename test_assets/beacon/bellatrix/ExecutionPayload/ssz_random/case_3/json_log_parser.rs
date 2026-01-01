use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, PartialEq)]
pub enum LogSeverity {
    Error,
    Warning,
    Info,
    Debug,
}

pub fn parse_log_line(line: &str) -> Option<(LogSeverity, String)> {
    let parsed: Value = serde_json::from_str(line).ok()?;
    let severity_str = parsed.get("severity")?.as_str()?;
    let message = parsed.get("message")?.as_str()?;

    let severity = match severity_str.to_lowercase().as_str() {
        "error" => LogSeverity::Error,
        "warning" => LogSeverity::Warning,
        "info" => LogSeverity::Info,
        "debug" => LogSeverity::Debug,
        _ => return None,
    };

    Some((severity, message.to_string()))
}

pub fn filter_logs_by_severity(
    file_path: &str,
    min_severity: LogSeverity,
) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut filtered_logs = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Some((severity, message)) = parse_log_line(&line) {
            if severity_priority(&severity) <= severity_priority(&min_severity) {
                filtered_logs.push(format!("{:?}: {}", severity, message));
            }
        }
    }

    Ok(filtered_logs)
}

fn severity_priority(severity: &LogSeverity) -> u8 {
    match severity {
        LogSeverity::Error => 1,
        LogSeverity::Warning => 2,
        LogSeverity::Info => 3,
        LogSeverity::Debug => 4,
    }
}