use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
struct LogFilter {
    min_level: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    keyword: Option<String>,
}

impl LogFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let levels = vec!["trace", "debug", "info", "warn", "error"];
            let entry_idx = levels.iter().position(|&l| l == entry.level.to_lowercase());
            let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());
            
            if let (Some(e_idx), Some(m_idx)) = (entry_idx, min_idx) {
                if e_idx < m_idx {
                    return false;
                }
            }
        }

        if let (Some(start), Some(end)) = (&self.start_time, &self.end_time) {
            if let Ok(entry_time) = DateTime::parse_from_rfc3339(&entry.timestamp) {
                let entry_utc = entry_time.with_timezone(&Utc);
                if entry_utc < *start || entry_utc > *end {
                    return false;
                }
            }
        }

        if let Some(keyword) = &self.keyword {
            if !entry.message.contains(keyword) {
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
            Err(e) => {
                eprintln!("Failed to parse line: {}, error: {}", line, e);
            }
        }
    }

    Ok(entries)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = LogFilter {
        min_level: Some("info".to_string()),
        start_time: Some(Utc::now() - chrono::Duration::hours(24)),
        end_time: Some(Utc::now()),
        keyword: Some("connection".to_string()),
    };

    let entries = parse_log_file("application.log", &filter)?;
    
    println!("Found {} matching log entries:", entries.len());
    for entry in entries.iter().take(5) {
        println!("[{}] {}: {}", entry.timestamp, entry.level, entry.message);
    }

    Ok(())
}