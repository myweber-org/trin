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
pub struct LogSummary {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    level_distribution: HashMap<String, usize>,
}

impl LogEntry {
    pub fn parse(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() == 3 {
            Some(LogEntry {
                timestamp: parts[0].trim().to_string(),
                level: parts[1].trim().to_string(),
                message: parts[2].trim().to_string(),
            })
        } else {
            None
        }
    }
}

pub fn analyze_log_file<P: AsRef<Path>>(path: P) -> std::io::Result<LogSummary> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut summary = LogSummary {
        total_entries: 0,
        error_count: 0,
        warning_count: 0,
        info_count: 0,
        level_distribution: HashMap::new(),
    };

    for line in reader.lines() {
        let line = line?;
        if let Some(entry) = LogEntry::parse(&line) {
            summary.total_entries += 1;
            *summary.level_distribution.entry(entry.level.clone()).or_insert(0) += 1;

            match entry.level.as_str() {
                "ERROR" => summary.error_count += 1,
                "WARNING" => summary.warning_count += 1,
                "INFO" => summary.info_count += 1,
                _ => {}
            }
        }
    }

    Ok(summary)
}

pub fn display_summary(summary: &LogSummary) {
    println!("Log Analysis Summary:");
    println!("Total entries: {}", summary.total_entries);
    println!("Errors: {}", summary.error_count);
    println!("Warnings: {}", summary.warning_count);
    println!("Info messages: {}", summary.info_count);
    
    println!("\nLevel distribution:");
    for (level, count) in &summary.level_distribution {
        println!("  {}: {}", level, count);
    }
}