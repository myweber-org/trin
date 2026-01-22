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

struct LogAnalyzer {
    entries: Vec<LogEntry>,
    stats: HashMap<String, usize>,
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            stats: HashMap::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    fn analyze(&mut self) {
        self.stats.clear();
        for entry in &self.entries {
            *self.stats.entry(entry.level.clone()).or_insert(0) += 1;
            *self.stats.entry(entry.service.clone()).or_insert(0) += 1;
        }
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    fn generate_summary(&self) -> HashMap<String, usize> {
        self.stats.clone()
    }

    fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = LogAnalyzer::new();
    
    analyzer.load_from_file("logs.jsonl")?;
    analyzer.analyze();

    println!("Total entries: {}", analyzer.entries.len());
    
    let summary = analyzer.generate_summary();
    println!("Summary: {:?}", summary);

    let errors = analyzer.filter_by_level("ERROR");
    println!("Error count: {}", errors.len());

    let api_logs = analyzer.filter_by_service("api-service");
    println!("API service logs: {}", api_logs.len());

    let search_results = analyzer.search_messages("timeout");
    println!("Timeout occurrences: {}", search_results.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_initialization() {
        let analyzer = LogAnalyzer::new();
        assert_eq!(analyzer.entries.len(), 0);
        assert_eq!(analyzer.stats.len(), 0);
    }

    #[test]
    fn test_filtering() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.entries.push(LogEntry {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            level: "ERROR".to_string(),
            service: "api-service".to_string(),
            message: "Connection timeout".to_string(),
            extra: HashMap::new(),
        });

        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
    }
}