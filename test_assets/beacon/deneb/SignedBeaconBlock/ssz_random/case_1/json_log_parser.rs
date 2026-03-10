use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(Debug)]
struct LogFilter {
    min_level: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let levels = ["trace", "debug", "info", "warn", "error"];
            let entry_level_idx = levels.iter().position(|&l| l == entry.level.to_lowercase());
            let min_level_idx = levels.iter().position(|&l| l == min_level.to_lowercase());

            match (entry_level_idx, min_level_idx) {
                (Some(e_idx), Some(m_idx)) if e_idx < m_idx => return false,
                _ => {}
            }
        }

        if let (Some(start), Some(end)) = (&self.start_time, &self.end_time) {
            if let Ok(entry_time) = entry.timestamp.parse::<DateTime<Utc>>() {
                if entry_time < *start || entry_time > *end {
                    return false;
                }
            }
        }

        true
    }
}

fn parse_log_file(path: &str, filter: &LogFilter) -> Result<Vec<LogEntry>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) if filter.matches(&entry) => entries.push(entry),
            Ok(_) => continue,
            Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
        }
    }

    Ok(entries)
}

fn main() -> Result<(), Box<dyn Error>> {
    let filter = LogFilter {
        min_level: Some("info".to_string()),
        start_time: Some("2024-01-01T00:00:00Z".parse::<DateTime<Utc>>()?),
        end_time: Some("2024-12-31T23:59:59Z".parse::<DateTime<Utc>>()?),
    };

    let entries = parse_log_file("app.log", &filter)?;
    
    for entry in entries {
        println!(
            "[{}] {}: {}",
            entry.timestamp, entry.level, entry.message
        );
    }

    Ok(())
}