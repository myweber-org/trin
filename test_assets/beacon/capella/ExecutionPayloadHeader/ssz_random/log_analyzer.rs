use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    error_patterns: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            error_patterns: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, filepath: &str) -> Result<(), std::io::Error> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)").unwrap();
        let error_pattern = Regex::new(r"error|failed|exception|timeout", Regex::case_insensitive).unwrap();

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

                if error_pattern.is_match(&message) {
                    let error_key = message.split_whitespace().next().unwrap_or("unknown").to_string();
                    *self.error_patterns.entry(error_key).or_insert(0) += 1;
                }
            }
        }

        Ok(())
    }

    pub fn get_summary(&self) -> String {
        let total_entries = self.entries.len();
        let error_count = self.level_counts.get("ERROR").unwrap_or(&0);
        let warning_count = self.level_counts.get("WARN").unwrap_or(&0);
        let info_count = self.level_counts.get("INFO").unwrap_or(&0);

        let mut summary = format!(
            "Log Analysis Summary:\n\
            Total entries: {}\n\
            ERROR level: {}\n\
            WARN level: {}\n\
            INFO level: {}\n\n\
            Common error patterns:\n",
            total_entries, error_count, warning_count, info_count
        );

        let mut sorted_errors: Vec<_> = self.error_patterns.iter().collect();
        sorted_errors.sort_by(|a, b| b.1.cmp(a.1));

        for (pattern, count) in sorted_errors.iter().take(5) {
            summary.push_str(&format!("{}: {}\n", pattern, count));
        }

        summary
    }

    pub fn find_entries_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_timeline(&self) -> Vec<String> {
        let mut timeline = Vec::new();
        let mut current_hour = String::new();
        let mut hour_count = 0;

        for entry in &self.entries {
            let hour = entry.timestamp[..13].to_string();
            if hour != current_hour {
                if !current_hour.is_empty() {
                    timeline.push(format!("{}: {} entries", current_hour, hour_count));
                }
                current_hour = hour;
                hour_count = 1;
            } else {
                hour_count += 1;
            }
        }

        if !current_hour.is_empty() {
            timeline.push(format!("{}: {} entries", current_hour, hour_count));
        }

        timeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_analyzer_creation() {
        let analyzer = LogAnalyzer::new();
        assert_eq!(analyzer.entries.len(), 0);
        assert_eq!(analyzer.level_counts.len(), 0);
    }

    #[test]
    fn test_summary_generation() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file("test.log").unwrap();
        let summary = analyzer.get_summary();
        assert!(summary.contains("Log Analysis Summary"));
    }
}