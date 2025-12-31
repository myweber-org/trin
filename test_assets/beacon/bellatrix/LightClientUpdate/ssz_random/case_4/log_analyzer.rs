use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

#[derive(Debug)]
pub struct LogStats {
    total_entries: usize,
    level_counts: HashMap<String, usize>,
    error_messages: Vec<String>,
}

pub fn analyze_log_file<P: AsRef<Path>>(path: P) -> Result<LogStats, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut stats = LogStats {
        total_entries: 0,
        level_counts: HashMap::new(),
        error_messages: Vec::new(),
    };

    for line in reader.lines() {
        let line = line?;
        if let Some(entry) = parse_log_line(&line) {
            stats.total_entries += 1;
            *stats.level_counts.entry(entry.level.clone()).or_insert(0) += 1;

            if entry.level == "ERROR" {
                stats.error_messages.push(entry.message.clone());
            }
        }
    }

    Ok(stats)
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return None;
    }

    Some(LogEntry {
        timestamp: parts[0].to_string(),
        level: parts[1].to_string(),
        message: parts[2].to_string(),
    })
}

pub fn print_summary(stats: &LogStats) {
    println!("Log Analysis Summary");
    println!("====================");
    println!("Total entries: {}", stats.total_entries);
    println!("\nLevel distribution:");
    for (level, count) in &stats.level_counts {
        println!("  {}: {}", level, count);
    }

    if !stats.error_messages.is_empty() {
        println!("\nError messages found ({}):", stats.error_messages.len());
        for (i, msg) in stats.error_messages.iter().enumerate() {
            println!("  {}. {}", i + 1, msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_line() {
        let line = "2024-01-15T10:30:00 INFO System started";
        let entry = parse_log_line(line).unwrap();
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "System started");
    }

    #[test]
    fn test_parse_invalid_line() {
        let line = "Invalid log line";
        assert!(parse_log_line(line).is_none());
    }
}