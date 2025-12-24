use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

struct LogStats {
    total_entries: usize,
    level_counts: HashMap<String, usize>,
    service_counts: HashMap<String, usize>,
    error_messages: Vec<String>,
}

impl LogStats {
    fn new() -> Self {
        LogStats {
            total_entries: 0,
            level_counts: HashMap::new(),
            service_counts: HashMap::new(),
            error_messages: Vec::new(),
        }
    }

    fn update(&mut self, entry: &LogEntry) {
        self.total_entries += 1;

        *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        *self.service_counts.entry(entry.service.clone()).or_insert(0) += 1;

        if entry.level == "ERROR" {
            self.error_messages.push(entry.message.clone());
        }
    }

    fn display(&self) {
        println!("Total log entries: {}", self.total_entries);
        println!("\nLog level distribution:");
        for (level, count) in &self.level_counts {
            println!("  {}: {}", level, count);
        }

        println!("\nService distribution:");
        for (service, count) in &self.service_counts {
            println!("  {}: {}", service, count);
        }

        if !self.error_messages.is_empty() {
            println!("\nError messages ({} total):", self.error_messages.len());
            for msg in &self.error_messages {
                println!("  - {}", msg);
            }
        }
    }
}

fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Failed to parse line: {}\nError: {}", line, e),
        }
    }

    Ok(entries)
}

fn filter_logs_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level == level)
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = "logs/app.log";
    
    println!("Parsing log file: {}", log_file);
    let entries = parse_log_file(log_file)?;

    let mut stats = LogStats::new();
    for entry in &entries {
        stats.update(entry);
    }

    stats.display();

    let error_logs = filter_logs_by_level(&entries, "ERROR");
    println!("\nFound {} ERROR logs", error_logs.len());

    if !error_logs.is_empty() {
        println!("\nSample error log:");
        println!("{:#?}", error_logs[0]);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_logs_by_level() {
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-01T10:00:00Z".to_string(),
                level: "INFO".to_string(),
                service: "api".to_string(),
                message: "Service started".to_string(),
                extra: HashMap::new(),
            },
            LogEntry {
                timestamp: "2024-01-01T10:01:00Z".to_string(),
                level: "ERROR".to_string(),
                service: "api".to_string(),
                message: "Connection failed".to_string(),
                extra: HashMap::new(),
            },
        ];

        let errors = filter_logs_by_level(&entries, "ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Connection failed");
    }

    #[test]
    fn test_log_stats() {
        let mut stats = LogStats::new();
        
        let entry = LogEntry {
            timestamp: "2024-01-01T10:00:00Z".to_string(),
            level: "ERROR".to_string(),
            service: "api".to_string(),
            message: "Test error".to_string(),
            extra: HashMap::new(),
        };

        stats.update(&entry);
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.level_counts.get("ERROR"), Some(&1));
        assert_eq!(stats.service_counts.get("api"), Some(&1));
        assert_eq!(stats.error_messages.len(), 1);
    }
}