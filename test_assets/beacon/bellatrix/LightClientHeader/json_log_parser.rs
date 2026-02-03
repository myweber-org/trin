use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
    component: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

struct LogFilter {
    min_level: LogLevel,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    component_filter: Option<String>,
}

impl LogFilter {
    fn new(min_level: LogLevel) -> Self {
        LogFilter {
            min_level,
            start_time: None,
            end_time: None,
            component_filter: None,
        }
    }

    fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    fn with_component(mut self, component: &str) -> Self {
        self.component_filter = Some(component.to_string());
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

        if let Some(ref filter_component) = self.component_filter {
            if !entry.component.contains(filter_component) {
                return false;
            }
        }

        true
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
            Err(e) => eprintln!("Failed to parse line: {}, error: {}", line, e),
        }
    }

    Ok(entries)
}

fn filter_logs(entries: Vec<LogEntry>, filter: &LogFilter) -> Vec<LogEntry> {
    entries.into_iter().filter(|e| filter.matches(e)).collect()
}

fn analyze_logs(entries: &[LogEntry]) {
    let mut level_counts = std::collections::HashMap::new();
    let mut component_counts = std::collections::HashMap::new();

    for entry in entries {
        *level_counts.entry(&entry.level).or_insert(0) += 1;
        *component_counts.entry(&entry.component).or_insert(0) += 1;
    }

    println!("Log Analysis:");
    println!("Total entries: {}", entries.len());
    println!("\nBy level:");
    for (level, count) in &level_counts {
        println!("  {:?}: {}", level, count);
    }
    println!("\nBy component:");
    for (component, count) in &component_counts {
        println!("  {}: {}", component, count);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entries = parse_log_file("application.log")?;
    
    let filter = LogFilter::new(LogLevel::INFO)
        .with_time_range(
            "2024-01-01T00:00:00Z".parse::<DateTime<Utc>>()?,
            "2024-12-31T23:59:59Z".parse::<DateTime<Utc>>()?
        )
        .with_component("database");

    let filtered = filter_logs(entries, &filter);
    analyze_logs(&filtered);

    if !filtered.is_empty() {
        println!("\nSample filtered entries:");
        for entry in filtered.iter().take(3) {
            println!("{:?}", entry);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_filter() {
        let entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap(),
            level: LogLevel::INFO,
            message: "Test message".to_string(),
            component: "database".to_string(),
            metadata: serde_json::Value::Null,
        };

        let filter = LogFilter::new(LogLevel::INFO);
        assert!(filter.matches(&entry));

        let filter = LogFilter::new(LogLevel::WARN);
        assert!(!filter.matches(&entry));
    }

    #[test]
    fn test_time_filter() {
        let entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap(),
            level: LogLevel::INFO,
            message: "Test".to_string(),
            component: "app".to_string(),
            metadata: serde_json::Value::Null,
        };

        let filter = LogFilter::new(LogLevel::INFO)
            .with_time_range(
                Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap()
            );

        assert!(filter.matches(&entry));
    }
}