
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(default)]
    error: Option<String>,
}

fn parse_log_file(file_path: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Failed to parse line: {} - Error: {}", line, e),
        }
    }

    Ok(entries)
}

fn filter_errors(entries: &[LogEntry]) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level == "ERROR" || entry.error.is_some())
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let entries = parse_log_file("application.log")?;
    let error_entries = filter_errors(&entries);

    println!("Found {} error entries:", error_entries.len());
    for entry in error_entries {
        println!(
            "[{}] {} - {}",
            entry.timestamp,
            entry.level,
            entry.message
        );
        if let Some(err) = &entry.error {
            println!("  Error detail: {}", err);
        }
    }

    Ok(())
}