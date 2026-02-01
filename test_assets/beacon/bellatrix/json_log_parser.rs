use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
    module: Option<String>,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

struct LogFilter {
    min_level: LogLevel,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    module_filter: Option<String>,
}

impl LogFilter {
    fn new(min_level: LogLevel) -> Self {
        Self {
            min_level,
            start_time: None,
            end_time: None,
            module_filter: None,
        }
    }

    fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    fn with_module(mut self, module: &str) -> Self {
        self.module_filter = Some(module.to_string());
        self
    }

    fn matches(&self, entry: &LogEntry) -> bool {
        if entry.level > self.min_level {
            return false;
        }

        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }

        if let Some(ref module_filter) = self.module_filter {
            if let Some(ref module) = entry.module {
                if module != module_filter {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

fn parse_log_file<P: AsRef<Path>>(path: P, filter: &LogFilter) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => {
                if filter.matches(&entry) {
                    entries.push(entry);
                }
            }
            Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
        }
    }

    Ok(entries)
}

fn analyze_logs(entries: &[LogEntry]) {
    let mut level_counts = std::collections::HashMap::new();
    let mut module_counts = std::collections::HashMap::new();

    for entry in entries {
        *level_counts.entry(&entry.level).or_insert(0) += 1;
        if let Some(ref module) = entry.module {
            *module_counts.entry(module.clone()).or_insert(0) += 1;
        }
    }

    println!("Log Analysis:");
    println!("Total entries: {}", entries.len());
    println!("\nBy level:");
    for (level, count) in &level_counts {
        println!("  {:?}: {}", level, count);
    }
    println!("\nBy module:");
    for (module, count) in &module_counts {
        println!("  {}: {}", module, count);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = LogFilter::new(LogLevel::INFO)
        .with_module("database")
        .with_time_range(
            Utc::now() - chrono::Duration::hours(24),
            Utc::now(),
        );

    let entries = parse_log_file("logs/app.log", &filter)?;
    analyze_logs(&entries);

    if let Some(last_error) = entries.iter().find(|e| e.level == LogLevel::ERROR) {
        println!("\nLast error found:");
        println!("  Time: {}", last_error.timestamp);
        println!("  Message: {}", last_error.message);
        if let Some(ref module) = last_error.module {
            println!("  Module: {}", module);
        }
    }

    Ok(())
}