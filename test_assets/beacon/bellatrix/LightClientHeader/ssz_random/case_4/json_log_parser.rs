use serde::{Deserialize, Serialize};
use std::error::Error;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    source: String,
    metadata: serde_json::Value,
}

struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    fn add_entry(&mut self, entry: LogEntry) {
        self.entries.push(entry);
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    fn filter_by_time_range(&self, start: &str, end: &str) -> Result<Vec<&LogEntry>, Box<dyn Error>> {
        let start_time: DateTime<Utc> = start.parse()?;
        let end_time: DateTime<Utc> = end.parse()?;

        let filtered: Vec<&LogEntry> = self.entries
            .iter()
            .filter(|entry| {
                if let Ok(entry_time) = entry.timestamp.parse::<DateTime<Utc>>() {
                    entry_time >= start_time && entry_time <= end_time
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered)
    }

    fn count_entries(&self) -> usize {
        self.entries.len()
    }

    fn get_level_distribution(&self) -> std::collections::HashMap<String, usize> {
        let mut distribution = std::collections::HashMap::new();
        
        for entry in &self.entries {
            *distribution.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        distribution
    }
}

fn create_sample_logs() -> Vec<LogEntry> {
    vec![
        LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: "INFO".to_string(),
            message: "Application started".to_string(),
            source: "server".to_string(),
            metadata: serde_json::json!({"pid": 1234, "version": "1.0.0"}),
        },
        LogEntry {
            timestamp: "2024-01-15T10:35:00Z".to_string(),
            level: "WARN".to_string(),
            message: "High memory usage detected".to_string(),
            source: "monitor".to_string(),
            metadata: serde_json::json!({"memory_mb": 2048, "threshold": 1800}),
        },
        LogEntry {
            timestamp: "2024-01-15T10:40:00Z".to_string(),
            level: "ERROR".to_string(),
            message: "Database connection failed".to_string(),
            source: "database".to_string(),
            metadata: serde_json::json!({"retry_count": 3, "timeout_sec": 30}),
        },
    ]
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut parser = LogParser::new();
    
    for log in create_sample_logs() {
        parser.add_entry(log);
    }
    
    println!("Total log entries: {}", parser.count_entries());
    
    let info_logs = parser.filter_by_level("INFO");
    println!("INFO logs count: {}", info_logs.len());
    
    let time_filtered = parser.filter_by_time_range(
        "2024-01-15T10:30:00Z",
        "2024-01-15T10:38:00Z"
    )?;
    println!("Logs in time range: {}", time_filtered.len());
    
    let distribution = parser.get_level_distribution();
    println!("Log level distribution:");
    for (level, count) in distribution {
        println!("  {}: {}", level, count);
    }
    
    Ok(())
}