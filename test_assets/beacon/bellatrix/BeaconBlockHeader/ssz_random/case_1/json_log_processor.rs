use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

struct LogFilter {
    min_level: String,
    service_pattern: Option<String>,
    keyword: Option<String>,
}

impl LogFilter {
    fn new(min_level: &str) -> Self {
        LogFilter {
            min_level: min_level.to_string(),
            service_pattern: None,
            keyword: None,
        }
    }

    fn with_service(mut self, pattern: &str) -> Self {
        self.service_pattern = Some(pattern.to_string());
        self
    }

    fn with_keyword(mut self, keyword: &str) -> Self {
        self.keyword = Some(keyword.to_string());
        self
    }

    fn matches(&self, entry: &LogEntry) -> bool {
        let level_order = |lvl: &str| match lvl {
            "ERROR" => 4,
            "WARN" => 3,
            "INFO" => 2,
            "DEBUG" => 1,
            _ => 0,
        };

        if level_order(&entry.level) < level_order(&self.min_level) {
            return false;
        }

        if let Some(ref pattern) = self.service_pattern {
            if !entry.service.contains(pattern) {
                return false;
            }
        }

        if let Some(ref keyword) = self.keyword {
            if !entry.message.contains(keyword) {
                return false;
            }
        }

        true
    }
}

fn process_log_file<P: AsRef<Path>>(
    path: P,
    filter: &LogFilter,
) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut results = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) if filter.matches(&entry) => results.push(entry),
            Ok(_) => continue,
            Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
        }
    }

    Ok(results)
}

fn export_to_json(entries: &[LogEntry], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    serde_json::to_writer_pretty(file, entries)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = LogFilter::new("INFO")
        .with_service("api")
        .with_keyword("timeout");

    let entries = process_log_file("logs/app.log", &filter)?;
    
    println!("Found {} matching log entries", entries.len());
    
    for entry in &entries {
        println!("[{}] {} - {}", entry.timestamp, entry.level, entry.message);
    }

    export_to_json(&entries, "filtered_logs.json")?;
    println!("Results exported to filtered_logs.json");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matching() {
        let entry = LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: "ERROR".to_string(),
            service: "api-gateway".to_string(),
            message: "Request timeout occurred".to_string(),
            metadata: None,
        };

        let filter = LogFilter::new("WARN")
            .with_service("api")
            .with_keyword("timeout");

        assert!(filter.matches(&entry));
    }

    #[test]
    fn test_filter_level_exclusion() {
        let entry = LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: "INFO".to_string(),
            service: "api-gateway".to_string(),
            message: "Request processed".to_string(),
            metadata: None,
        };

        let filter = LogFilter::new("ERROR");
        assert!(!filter.matches(&entry));
    }
}