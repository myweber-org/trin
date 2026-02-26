
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};
use regex::Regex;

pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
    pub source: String,
}

pub struct LogProcessor {
    pub min_level: String,
    pub keyword_filter: Option<String>,
}

impl LogProcessor {
    pub fn new(min_level: &str) -> Self {
        LogProcessor {
            min_level: min_level.to_lowercase(),
            keyword_filter: None,
        }
    }

    pub fn with_keyword_filter(mut self, keyword: &str) -> Self {
        self.keyword_filter = Some(keyword.to_lowercase());
        self
    }

    pub fn parse_log_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        let timestamp_re = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}[+-]\d{4})\]")?;
        let level_re = Regex::new(r"\b(ERROR|WARN|INFO|DEBUG|TRACE)\b")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line, &timestamp_re, &level_re) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str, timestamp_re: &Regex, level_re: &Regex) -> Option<LogEntry> {
        let timestamp_caps = timestamp_re.captures(line)?;
        let timestamp_str = timestamp_caps.get(1)?.as_str();
        let timestamp = DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S%z").ok()?;

        let level_caps = level_re.captures(line)?;
        let level = level_caps.get(1)?.as_str().to_lowercase();

        if !self.is_level_allowed(&level) {
            return None;
        }

        let message_start = timestamp_caps.get(0)?.end();
        let message = line[message_start..].trim().to_string();

        if let Some(ref keyword) = self.keyword_filter {
            if !message.to_lowercase().contains(keyword) {
                return None;
            }
        }

        Some(LogEntry {
            timestamp,
            level,
            message,
            source: line.to_string(),
        })
    }

    fn is_level_allowed(&self, level: &str) -> bool {
        let level_order = vec!["trace", "debug", "info", "warn", "error"];
        let min_index = level_order.iter().position(|&l| l == self.min_level).unwrap_or(0);
        let current_index = level_order.iter().position(|&l| l == level).unwrap_or(0);
        current_index >= min_index
    }

    pub fn generate_summary(&self, entries: &[LogEntry]) -> String {
        let mut counts = std::collections::HashMap::new();
        for entry in entries {
            *counts.entry(&entry.level).or_insert(0) += 1;
        }

        let mut summary = format!("Total entries: {}\n", entries.len());
        let levels = ["error", "warn", "info", "debug", "trace"];
        for level in levels.iter() {
            if let Some(count) = counts.get(level) {
                summary.push_str(&format!("{}: {}\n", level.to_uppercase(), count));
            }
        }

        if let Some(earliest) = entries.iter().map(|e| e.timestamp).min() {
            if let Some(latest) = entries.iter().map(|e| e.timestamp).max() {
                let duration = latest - earliest;
                summary.push_str(&format!(
                    "Time range: {} to {} (duration: {} seconds)\n",
                    earliest.format("%Y-%m-%d %H:%M:%S"),
                    latest.format("%Y-%m-%d %H:%M:%S"),
                    duration.num_seconds()
                ));
            }
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
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "[2024-01-15 10:30:45+0000] INFO Application started").unwrap();
        writeln!(temp_file, "[2024-01-15 10:31:00+0000] ERROR Database connection failed").unwrap();
        writeln!(temp_file, "[2024-01-15 10:32:15+0000] DEBUG Processing request").unwrap();

        let processor = LogProcessor::new("info");
        let entries = processor.parse_log_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "info");
        assert_eq!(entries[1].level, "error");
    }

    #[test]
    fn test_level_filtering() {
        let processor = LogProcessor::new("warn");
        assert!(processor.is_level_allowed("error"));
        assert!(processor.is_level_allowed("warn"));
        assert!(!processor.is_level_allowed("info"));
    }
}