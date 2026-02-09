use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    component: String,
    message: String,
    metadata: HashMap<String, String>,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    stats: AnalysisStats,
}

#[derive(Debug, Default)]
pub struct AnalysisStats {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    component_distribution: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            stats: AnalysisStats::default(),
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

        self.update_stats();
        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let timestamp_str = parts[0].trim();
        let level = parts[1].trim().to_string();
        let component = parts[2].trim().to_string();
        let message = parts[3].trim().to_string();

        let timestamp = DateTime::parse_from_rfc3339(timestamp_str).ok()?;

        let mut metadata = HashMap::new();
        if let Some((msg, meta)) = message.split_once(" - ") {
            for pair in meta.split(',') {
                if let Some((key, value)) = pair.split_once('=') {
                    metadata.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        Some(LogEntry {
            timestamp,
            level,
            component,
            message,
            metadata,
        })
    }

    fn update_stats(&mut self) {
        self.stats.total_entries = self.entries.len();
        self.stats.error_count = self.entries.iter()
            .filter(|e| e.level == "ERROR")
            .count();
        self.stats.warning_count = self.entries.iter()
            .filter(|e| e.level == "WARN")
            .count();

        self.stats.component_distribution.clear();
        for entry in &self.entries {
            *self.stats.component_distribution
                .entry(entry.component.clone())
                .or_insert(0) += 1;
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|e| e.level == level)
            .collect()
    }

    pub fn filter_by_component(&self, component: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|e| e.component == component)
            .collect()
    }

    pub fn get_stats(&self) -> &AnalysisStats {
        &self.stats
    }

    pub fn find_pattern(&self, pattern: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|e| e.message.contains(pattern))
            .collect()
    }
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_parsing() {
        let analyzer = LogAnalyzer::new();
        let line = "2024-01-15T10:30:00+00:00 | ERROR | Database | Connection failed - retry=3, timeout=30";
        
        let entry = analyzer.parse_log_line(line).unwrap();
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.component, "Database");
        assert_eq!(entry.metadata.get("retry").unwrap(), "3");
    }

    #[test]
    fn test_invalid_log_line() {
        let analyzer = LogAnalyzer::new();
        let line = "Invalid log format";
        
        assert!(analyzer.parse_log_line(line).is_none());
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    error_patterns: HashMap<String, usize>,
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            error_patterns: HashMap::new(),
        }
    }

    fn parse_log_file(&mut self, file_path: &str) -> io::Result<()> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+): (.+)").unwrap();
        let error_pattern = Regex::new(r"error|failed|exception|timeout",).unwrap();

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp = captures[1].to_string();
                let level = captures[2].to_string();
                let message = captures[3].to_string();

                let entry = LogEntry {
                    timestamp,
                    level: level.clone(),
                    message: message.clone(),
                };

                self.entries.push(entry);
                *self.level_counts.entry(level).or_insert(0) += 1;

                if error_pattern.is_match(&message.to_lowercase()) {
                    let error_key = message.split_whitespace().next().unwrap_or("unknown").to_string();
                    *self.error_patterns.entry(error_key).or_insert(0) += 1;
                }
            }
        }
        Ok(())
    }

    fn generate_report(&self) {
        println!("Log Analysis Report");
        println!("===================");
        println!("Total entries: {}", self.entries.len());
        println!("\nLog Level Distribution:");
        for (level, count) in &self.level_counts {
            println!("  {}: {}", level, count);
        }
        println!("\nCommon Error Patterns:");
        let mut sorted_errors: Vec<_> = self.error_patterns.iter().collect();
        sorted_errors.sort_by(|a, b| b.1.cmp(a.1));
        for (pattern, count) in sorted_errors.iter().take(5) {
            println!("  {}: {}", pattern, count);
        }
        println!("\nRecent Critical Entries:");
        let critical_entries: Vec<_> = self.entries
            .iter()
            .filter(|e| e.level == "ERROR" || e.level == "FATAL")
            .rev()
            .take(3)
            .collect();
        for entry in critical_entries {
            println!("  [{}] {}: {}", entry.timestamp, entry.level, entry.message);
        }
    }
}

fn main() {
    let mut analyzer = LogAnalyzer::new();
    
    if let Err(e) = analyzer.parse_log_file("application.log") {
        eprintln!("Failed to parse log file: {}", e);
        return;
    }
    
    analyzer.generate_report();
}