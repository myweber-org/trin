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

    fn get_summary(&self) -> String {
        let total_entries = self.entries.len();
        let level_counts = self.count_by_level();
        let service_counts = self.count_by_service();

        let mut summary = format!("Total log entries: {}\n", total_entries);
        
        summary.push_str("\nEntries by level:\n");
        for (level, count) in &level_counts {
            summary.push_str(&format!("  {}: {}\n", level, count));
        }

        summary.push_str("\nEntries by service:\n");
        for (service, count) in &service_counts {
            summary.push_str(&format!("  {}: {}\n", service, count));
        }

        summary
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    
    parser.load_from_file("logs.jsonl")?;
    
    println!("{}", parser.get_summary());
    
    let error_logs = parser.filter_by_level("error");
    println!("\nError logs ({} found):", error_logs.len());
    for log in error_logs.iter().take(3) {
        println!("  [{}] {}: {}", log.timestamp, log.service, log.message);
    }
    
    let search_results = parser.search_messages("timeout");
    println!("\nLogs containing 'timeout' ({} found):", search_results.len());
    for log in search_results.iter().take(3) {
        println!("  [{}] {} {}: {}", log.timestamp, log.level, log.service, log.message);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parser() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request processed","metadata":{"method":"GET","path":"/health"}}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"database","message":"Connection timeout","metadata":{"retry_count":"3"}}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"api","message":"High latency detected","metadata":{"latency_ms":"1500"}}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let mut parser = LogParser::new();
        parser.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(parser.entries.len(), 3);
        assert_eq!(parser.filter_by_level("ERROR").len(), 1);
        assert_eq!(parser.filter_by_service("api").len(), 2);
        assert_eq!(parser.search_messages("timeout").len(), 1);
        
        let counts = parser.count_by_level();
        assert_eq!(counts.get("INFO"), Some(&1));
        assert_eq!(counts.get("ERROR"), Some(&1));
    }
}