use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
    #[serde(flatten)]
    extra_fields: HashMap<String, serde_json::Value>,
}

struct LogFilter {
    min_level: Option<String>,
    contains_text: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let level_order = |lvl: &str| match lvl.to_lowercase().as_str() {
                "error" => 4,
                "warn" => 3,
                "info" => 2,
                "debug" => 1,
                "trace" => 0,
                _ => 0,
            };
            
            if level_order(&entry.level) < level_order(min_level) {
                return false;
            }
        }
        
        if let Some(text) = &self.contains_text {
            if !entry.message.contains(text) {
                return false;
            }
        }
        
        if let Some(start) = &self.start_time {
            if &entry.timestamp < start {
                return false;
            }
        }
        
        if let Some(end) = &self.end_time {
            if &entry.timestamp > end {
                return false;
            }
        }
        
        true
    }
}

fn parse_log_file(path: &str, filter: Option<LogFilter>) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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
                if let Some(ref filter) = filter {
                    if filter.matches(&entry) {
                        entries.push(entry);
                    }
                } else {
                    entries.push(entry);
                }
            }
            Err(e) => eprintln!("Failed to parse line: {}, error: {}", line, e),
        }
    }
    
    Ok(entries)
}

fn format_entry(entry: &LogEntry, show_extra: bool) -> String {
    let mut output = format!(
        "[{}] {}: {}",
        entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
        entry.level.to_uppercase(),
        entry.message
    );
    
    if show_extra && !entry.extra_fields.is_empty() {
        output.push_str(&format!(" | Extra: {:?}", entry.extra_fields));
    }
    
    output
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = LogFilter {
        min_level: Some("info".to_string()),
        contains_text: Some("error".to_string()),
        start_time: None,
        end_time: None,
    };
    
    let entries = parse_log_file("application.log", Some(filter))?;
    
    for entry in entries {
        println!("{}", format_entry(&entry, true));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    
    #[test]
    fn test_filter_matches() {
        let entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            level: "error".to_string(),
            message: "Database connection failed".to_string(),
            extra_fields: HashMap::new(),
        };
        
        let filter = LogFilter {
            min_level: Some("warn".to_string()),
            contains_text: Some("connection".to_string()),
            start_time: None,
            end_time: None,
        };
        
        assert!(filter.matches(&entry));
    }
    
    #[test]
    fn test_filter_rejects_lower_level() {
        let entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            level: "info".to_string(),
            message: "Application started".to_string(),
            extra_fields: HashMap::new(),
        };
        
        let filter = LogFilter {
            min_level: Some("warn".to_string()),
            contains_text: None,
            start_time: None,
            end_time: None,
        };
        
        assert!(!filter.matches(&entry));
    }
}