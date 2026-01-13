use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    component: Option<String>,
    trace_id: Option<String>,
}

#[derive(Debug)]
struct LogStats {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    components: Vec<String>,
}

impl LogEntry {
    fn from_json(line: &str) -> Result<Self, Box<dyn Error>> {
        let entry: LogEntry = serde_json::from_str(line)?;
        Ok(entry)
    }
}

fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        match LogEntry::from_json(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e),
        }
    }

    Ok(entries)
}

fn analyze_logs(entries: &[LogEntry]) -> LogStats {
    let mut stats = LogStats {
        total_entries: entries.len(),
        error_count: 0,
        warning_count: 0,
        components: Vec::new(),
    };

    let mut unique_components = std::collections::HashSet::new();

    for entry in entries {
        match entry.level.as_str() {
            "ERROR" => stats.error_count += 1,
            "WARN" => stats.warning_count += 1,
            _ => {}
        }

        if let Some(component) = &entry.component {
            unique_components.insert(component.clone());
        }
    }

    stats.components = unique_components.into_iter().collect();
    stats.components.sort();

    stats
}

fn filter_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level == level)
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <log_file.json>", args[0]);
        std::process::exit(1);
    }

    let log_file = &args[1];
    let entries = parse_log_file(log_file)?;
    
    println!("Successfully parsed {} log entries", entries.len());
    
    let stats = analyze_logs(&entries);
    println!("Log Statistics:");
    println!("  Total entries: {}", stats.total_entries);
    println!("  Errors: {}", stats.error_count);
    println!("  Warnings: {}", stats.warning_count);
    println!("  Components: {}", stats.components.join(", "));
    
    if stats.error_count > 0 {
        let errors = filter_by_level(&entries, "ERROR");
        println!("\nError entries:");
        for error in errors.iter().take(5) {
            println!("  [{}] {} - {}", error.timestamp, error.component.as_deref().unwrap_or("unknown"), error.message);
        }
        if errors.len() > 5 {
            println!("  ... and {} more errors", errors.len() - 5);
        }
    }
    
    Ok(())
}