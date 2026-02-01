use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    component: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer { entries: Vec::new() }
    }

    pub fn load_from_file(&mut self, path: &str) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"^(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}) ([A-Z]+) \[([^\]]+)\] (.+)$").unwrap();

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp_str = captures.get(1).unwrap().as_str();
                let level = captures.get(2).unwrap().as_str().to_string();
                let component = captures.get(3).unwrap().as_str().to_string();
                let message = captures.get(4).unwrap().as_str().to_string();

                let timestamp = DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S%.3f %z")
                    .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap());

                self.entries.push(LogEntry {
                    timestamp,
                    level,
                    component,
                    message,
                });
            }
        }
        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn count_by_component(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.component.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn find_errors_with_context(&self, search_term: &str) -> Vec<String> {
        let mut results = Vec::new();
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.message.contains(search_term) {
                let start = if i > 2 { i - 2 } else { 0 };
                let end = if i + 3 < self.entries.len() { i + 3 } else { self.entries.len() };
                
                let context: Vec<String> = self.entries[start..end]
                    .iter()
                    .map(|e| format!("{} {} [{}] {}", e.timestamp, e.level, e.component, e.message))
                    .collect();
                
                results.push(context.join("\n"));
            }
        }
        results
    }

    pub fn generate_summary(&self) -> String {
        let total = self.entries.len();
        let error_count = self.filter_by_level("ERROR").len();
        let warning_count = self.filter_by_level("WARN").len();
        let component_counts = self.count_by_component();

        let mut summary = format!("Total entries: {}\nErrors: {}\nWarnings: {}\n\nComponent distribution:\n", 
                                 total, error_count, warning_count);
        
        for (component, count) in component_counts {
            summary.push_str(&format!("{}: {} entries\n", component, count));
        }
        
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2024-01-15 10:30:45.123 ERROR [Database] Connection failed").unwrap();
        writeln!(temp_file, "2024-01-15 10:30:46.456 WARN [Network] High latency detected").unwrap();
        writeln!(temp_file, "2024-01-15 10:30:47.789 INFO [API] Request processed").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(analyzer.entries.len(), 3);
        assert_eq!(analyzer.filter_by_level("ERROR").len(), 1);
        assert_eq!(analyzer.filter_by_level("WARN").len(), 1);
        
        let counts = analyzer.count_by_component();
        assert_eq!(counts.get("Database"), Some(&1));
    }
}