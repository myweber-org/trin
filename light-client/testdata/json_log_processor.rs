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

impl LogEntry {
    fn is_error(&self) -> bool {
        self.level == "ERROR"
    }

    fn from_json_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

struct LogProcessor {
    entries: Vec<LogEntry>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = LogEntry::from_json_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
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

    fn error_count(&self) -> usize {
        self.entries.iter().filter(|entry| entry.is_error()).count()
    }

    fn unique_services(&self) -> Vec<&str> {
        let mut services: Vec<&str> = self.entries.iter().map(|e| e.service.as_str()).collect();
        services.sort();
        services.dedup();
        services
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LogProcessor::new();
    
    processor.load_from_file("logs/app.log")?;
    
    println!("Total entries: {}", processor.entries.len());
    println!("Error count: {}", processor.error_count());
    println!("Unique services: {:?}", processor.unique_services());
    
    let errors = processor.filter_by_level("ERROR");
    println!("Recent errors:");
    for error in errors.iter().take(5) {
        println!("  [{}] {}: {}", error.timestamp, error.service, error.message);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_parsing() {
        let json = r#"{
            "timestamp": "2023-10-05T14:30:00Z",
            "level": "INFO",
            "service": "api",
            "message": "Request processed",
            "metadata": {"user_id": 123}
        }"#;
        
        let entry = LogEntry::from_json_line(json).unwrap();
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.service, "api");
        assert!(!entry.is_error());
    }

    #[test]
    fn test_error_detection() {
        let error_entry = LogEntry {
            timestamp: "2023-10-05T14:30:00Z".to_string(),
            level: "ERROR".to_string(),
            service: "database".to_string(),
            message: "Connection failed".to_string(),
            metadata: None,
        };
        
        assert!(error_entry.is_error());
    }
}