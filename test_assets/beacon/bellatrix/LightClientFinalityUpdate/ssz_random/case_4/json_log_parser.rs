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
    metadata: HashMap<String, String>,
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

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)?;
            self.entries.push(entry);
        }

        Ok(())
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn count_by_service(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.service.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }

    fn get_time_range(&self) -> Option<(String, String)> {
        if self.entries.is_empty() {
            return None;
        }

        let mut timestamps: Vec<&String> = self.entries.iter().map(|e| &e.timestamp).collect();
        timestamps.sort();

        Some((timestamps[0].clone(), timestamps[timestamps.len() - 1].clone()))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    
    parser.load_from_file("logs.jsonl")?;
    
    println!("Total log entries: {}", parser.entries.len());
    
    let error_logs = parser.filter_by_level("error");
    println!("Error logs: {}", error_logs.len());
    
    let counts = parser.count_by_level();
    println!("Log level distribution:");
    for (level, count) in counts {
        println!("  {}: {}", level, count);
    }
    
    if let Some((start, end)) = parser.get_time_range() {
        println!("Time range: {} to {}", start, end);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request processed","metadata":{"user_id":"123"}}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"database","message":"Connection failed","metadata":{"retry_count":"3"}}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"api","message":"Slow response","metadata":{"duration_ms":"1500"}}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let mut parser = LogParser::new();
        parser.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(parser.entries.len(), 3);
        assert_eq!(parser.filter_by_level("error").len(), 1);
        assert_eq!(parser.filter_by_service("api").len(), 2);
        
        let counts = parser.count_by_level();
        assert_eq!(counts.get("INFO"), Some(&1));
        assert_eq!(counts.get("ERROR"), Some(&1));
        assert_eq!(counts.get("WARN"), Some(&1));
    }
}