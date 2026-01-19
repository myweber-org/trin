use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    message: String,
    metadata: HashMap<String, String>,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() < 4 {
            return None;
        }

        let timestamp_str = format!("{} {}", parts[0], parts[1]);
        let timestamp = DateTime::parse_from_str(&timestamp_str, "%Y-%m-%d %H:%M:%S %z").ok()?;

        let level = parts[2].to_string();
        let message = parts[3].to_string();

        let mut metadata = HashMap::new();
        if let Some(meta_start) = message.find('{') {
            if let Some(meta_end) = message.find('}') {
                let meta_str = &message[meta_start + 1..meta_end];
                for pair in meta_str.split(',') {
                    let kv: Vec<&str> = pair.split('=').collect();
                    if kv.len() == 2 {
                        metadata.insert(kv[0].trim().to_string(), kv[1].trim().to_string());
                    }
                }
            }
        }

        Some(LogEntry {
            timestamp,
            level,
            message,
            metadata,
        })
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn find_errors(&self) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == "ERROR" || entry.level == "FATAL")
            .collect()
    }

    pub fn time_range(&self) -> Option<(DateTime<FixedOffset>, DateTime<FixedOffset>)> {
        if self.entries.is_empty() {
            return None;
        }

        let mut min_time = &self.entries[0].timestamp;
        let mut max_time = &self.entries[0].timestamp;

        for entry in &self.entries {
            if entry.timestamp < *min_time {
                min_time = &entry.timestamp;
            }
            if entry.timestamp > *max_time {
                max_time = &entry.timestamp;
            }
        }

        Some((*min_time, *max_time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut analyzer = LogAnalyzer::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "2023-10-01 12:00:00 +0000 INFO Application started {{user=admin,ip=127.0.0.1}}").unwrap();
        writeln!(temp_file, "2023-10-01 12:01:00 +0000 ERROR Database connection failed {{db=primary,retry=3}}").unwrap();
        
        analyzer.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(analyzer.entries.len(), 2);
        assert_eq!(analyzer.count_by_level()["INFO"], 1);
        assert_eq!(analyzer.count_by_level()["ERROR"], 1);
    }

    #[test]
    fn test_error_filtering() {
        let mut analyzer = LogAnalyzer::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "2023-10-01 12:00:00 +0000 INFO Normal operation").unwrap();
        writeln!(temp_file, "2023-10-01 12:01:00 +0000 ERROR Something went wrong").unwrap();
        writeln!(temp_file, "2023-10-01 12:02:00 +0000 FATAL System crash").unwrap();
        
        analyzer.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        let errors = analyzer.find_errors();
        assert_eq!(errors.len(), 2);
    }
}