
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use chrono::{DateTime, FixedOffset};

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
    pub source: String,
}

pub struct LogProcessor {
    min_level: String,
    start_time: Option<DateTime<FixedOffset>>,
    end_time: Option<DateTime<FixedOffset>>,
    keyword_filter: Option<String>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            min_level: "INFO".to_string(),
            start_time: None,
            end_time: None,
            keyword_filter: None,
        }
    }

    pub fn set_min_level(&mut self, level: &str) -> &mut Self {
        self.min_level = level.to_uppercase();
        self
    }

    pub fn set_time_range(&mut self, start: Option<DateTime<FixedOffset>>, end: Option<DateTime<FixedOffset>>) -> &mut Self {
        self.start_time = start;
        self.end_time = end;
        self
    }

    pub fn set_keyword_filter(&mut self, keyword: Option<String>) -> &mut Self {
        self.keyword_filter = keyword;
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<LogEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                if self.filter_entry(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let timestamp_str = parts[0].trim();
        let level = parts[1].trim().to_string();
        let source = parts[2].trim().to_string();
        let message = parts[3].trim().to_string();

        match DateTime::parse_from_rfc3339(timestamp_str) {
            Ok(timestamp) => Some(LogEntry {
                timestamp,
                level,
                message,
                source,
            }),
            Err(_) => None,
        }
    }

    fn filter_entry(&self, entry: &LogEntry) -> bool {
        let level_order = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"];
        let entry_level_idx = level_order.iter().position(|&l| l == entry.level);
        let min_level_idx = level_order.iter().position(|&l| l == self.min_level);

        if let (Some(e_idx), Some(m_idx)) = (entry_level_idx, min_level_idx) {
            if e_idx < m_idx {
                return false;
            }
        }

        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }

        if let Some(ref keyword) = self.keyword_filter {
            if !entry.message.contains(keyword) && !entry.source.contains(keyword) {
                return false;
            }
        }

        true
    }

    pub fn write_filtered_logs<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> io::Result<usize> {
        let entries = self.process_file(input_path)?;
        let mut file = File::create(output_path)?;

        for entry in &entries {
            writeln!(
                file,
                "{} | {} | {} | {}",
                entry.timestamp.to_rfc3339(),
                entry.level,
                entry.source,
                entry.message
            )?;
        }

        Ok(entries.len())
    }
}

pub fn analyze_log_distribution(entries: &[LogEntry]) -> Vec<(String, usize)> {
    let mut distribution = std::collections::HashMap::new();
    
    for entry in entries {
        *distribution.entry(entry.level.clone()).or_insert(0) += 1;
    }
    
    let mut result: Vec<(String, usize)> = distribution.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}