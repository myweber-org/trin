use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
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
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }

    fn get_time_range(&self) -> Option<(&str, &str)> {
        if self.entries.is_empty() {
            return None;
        }

        let first = self.entries.first().unwrap();
        let last = self.entries.last().unwrap();
        Some((&first.timestamp, &last.timestamp))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    
    parser.load_from_file("logs.jsonl")?;
    
    println!("Total entries: {}", parser.entries.len());
    
    let error_logs = parser.filter_by_level("error");
    println!("Error logs: {}", error_logs.len());
    
    let counts = parser.count_by_level();
    for (level, count) in counts {
        println!("{}: {}", level, count);
    }
    
    if let Some((start, end)) = parser.get_time_range() {
        println!("Time range: {} - {}", start, end);
    }
    
    Ok(())
}