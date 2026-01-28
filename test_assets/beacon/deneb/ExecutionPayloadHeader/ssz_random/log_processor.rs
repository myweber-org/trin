
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use chrono::{DateTime, FixedOffset};
use regex::Regex;

pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
}

pub struct LogProcessor {
    pub entries: Vec<LogEntry>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let timestamp_re = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} [+-]\d{4})\]").unwrap();
        let level_re = Regex::new(r"\b(ERROR|WARN|INFO|DEBUG)\b").unwrap();

        for line in reader.lines() {
            let line = line?;
            if let Some(timestamp_caps) = timestamp_re.captures(&line) {
                if let Ok(timestamp) = DateTime::parse_from_str(&timestamp_caps[1], "%Y-%m-%d %H:%M:%S %z") {
                    let level = level_re.find(&line)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_else(|| "UNKNOWN".to_string());
                    
                    let message = line.clone();
                    
                    self.entries.push(LogEntry {
                        timestamp,
                        level,
                        message,
                    });
                }
            }
        }
        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_time_range(&self, start: DateTime<FixedOffset>, end: DateTime<FixedOffset>) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }

    pub fn count_by_level(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_parsing() {
        let mut processor = LogProcessor::new();
        let test_log = "[2024-01-15 14:30:00 +0000] INFO Application started\n[2024-01-15 14:35:00 +0000] ERROR Database connection failed";
        
        std::fs::write("test.log", test_log).unwrap();
        processor.load_from_file("test.log").unwrap();
        std::fs::remove_file("test.log").unwrap();

        assert_eq!(processor.entries.len(), 2);
        assert_eq!(processor.entries[0].level, "INFO");
        assert_eq!(processor.entries[1].level, "ERROR");
    }

    #[test]
    fn test_level_filtering() {
        let mut processor = LogProcessor::new();
        processor.entries.push(LogEntry {
            timestamp: FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2024, 1, 15, 14, 30, 0).unwrap(),
            level: "ERROR".to_string(),
            message: "Test error".to_string(),
        });
        
        let errors = processor.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
    }
}