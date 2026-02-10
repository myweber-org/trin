use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct LogProcessor {
    entries: Vec<LogEntry>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => {
                    self.entries.push(entry);
                    count += 1;
                }
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line, e),
            }
        }

        Ok(count)
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn group_by_service(&self) -> HashMap<String, Vec<&LogEntry>> {
        let mut groups: HashMap<String, Vec<&LogEntry>> = HashMap::new();
        
        for entry in &self.entries {
            groups
                .entry(entry.service.clone())
                .or_insert_with(Vec::new)
                .push(entry);
        }
        
        groups
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        counts
    }

    pub fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","service":"api","message":"Connection failed","metadata":{{"ip":"192.168.1.1","port":"8080"}}}}"#).unwrap();
        writeln!(file, r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","service":"auth","message":"User login successful","metadata":{{"user_id":"12345"}}}}"#).unwrap();
        writeln!(file, r#"{{"timestamp":"2024-01-15T10:32:00Z","level":"ERROR","service":"api","message":"Database timeout","metadata":{{"query":"SELECT * FROM users"}}}}"#).unwrap();
        writeln!(file, r#"{{"timestamp":"2024-01-15T10:33:00Z","level":"WARN","service":"cache","message":"Memory usage high","metadata":{{"usage":"85%"}}}}"#).unwrap();
        file
    }

    #[test]
    fn test_load_logs() {
        let mut processor = LogProcessor::new();
        let file = create_test_log_file();
        
        let count = processor.load_from_file(file.path()).unwrap();
        assert_eq!(count, 4);
        assert_eq!(processor.entries.len(), 4);
    }

    #[test]
    fn test_filter_by_level() {
        let mut processor = LogProcessor::new();
        let file = create_test_log_file();
        processor.load_from_file(file.path()).unwrap();
        
        let errors = processor.filter_by_level("ERROR");
        assert_eq!(errors.len(), 2);
        
        let infos = processor.filter_by_level("INFO");
        assert_eq!(infos.len(), 1);
    }

    #[test]
    fn test_group_by_service() {
        let mut processor = LogProcessor::new();
        let file = create_test_log_file();
        processor.load_from_file(file.path()).unwrap();
        
        let groups = processor.group_by_service();
        assert_eq!(groups.len(), 3);
        assert_eq!(groups.get("api").unwrap().len(), 2);
        assert_eq!(groups.get("auth").unwrap().len(), 1);
        assert_eq!(groups.get("cache").unwrap().len(), 1);
    }

    #[test]
    fn test_count_by_level() {
        let mut processor = LogProcessor::new();
        let file = create_test_log_file();
        processor.load_from_file(file.path()).unwrap();
        
        let counts = processor.count_by_level();
        assert_eq!(counts.get("ERROR").unwrap(), &2);
        assert_eq!(counts.get("INFO").unwrap(), &1);
        assert_eq!(counts.get("WARN").unwrap(), &1);
    }
}