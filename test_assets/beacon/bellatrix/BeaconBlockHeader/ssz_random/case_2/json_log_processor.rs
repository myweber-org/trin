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

struct LogProcessor {
    entries: Vec<LogEntry>,
    stats: HashMap<String, usize>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            stats: HashMap::new(),
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

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => {
                    self.update_stats(&entry);
                    self.entries.push(entry);
                }
                Err(e) => eprintln!("Failed to parse line: {} - Error: {}", line, e),
            }
        }

        Ok(())
    }

    fn update_stats(&mut self, entry: &LogEntry) {
        *self.stats.entry(entry.level.clone()).or_insert(0) += 1;
        *self.stats.entry(entry.service.clone()).or_insert(0) += 1;
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

    fn get_summary(&self) -> HashMap<String, usize> {
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
    let mut processor = LogProcessor::new();
    
    processor.load_from_file("logs.jsonl")?;
    
    println!("Total entries: {}", processor.entries.len());
    
    let errors = processor.filter_by_level("error");
    println!("Error entries: {}", errors.len());
    
    let summary = processor.get_summary();
    println!("Summary: {:?}", summary);
    
    let search_results = processor.search_messages("timeout");
    println!("Found {} entries with 'timeout'", search_results.len());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","service":"api","message":"Connection timeout","metadata":{"ip":"192.168.1.1"}}
{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","service":"auth","message":"User login successful","metadata":{"user_id":"123"}}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"api","message":"High latency detected","metadata":{}}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let mut processor = LogProcessor::new();
        processor.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(processor.entries.len(), 3);
        assert_eq!(processor.filter_by_level("error").len(), 1);
        assert_eq!(processor.filter_by_service("api").len(), 2);
        assert_eq!(processor.search_messages("timeout").len(), 1);
        
        let summary = processor.get_summary();
        assert_eq!(summary.get("ERROR"), Some(&1));
        assert_eq!(summary.get("api"), Some(&2));
    }
}