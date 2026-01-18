use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    source: String,
}

#[derive(Debug)]
pub struct LogStats {
    total_entries: usize,
    level_counts: HashMap<String, usize>,
    source_counts: HashMap<String, usize>,
    errors: Vec<LogEntry>,
    warnings: Vec<LogEntry>,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    stats: LogStats,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            stats: LogStats {
                total_entries: 0,
                level_counts: HashMap::new(),
                source_counts: HashMap::new(),
                errors: Vec::new(),
                warnings: Vec::new(),
            },
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.add_entry(entry);
            }
        }

        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() == 4 {
            Some(LogEntry {
                timestamp: parts[0].trim().to_string(),
                level: parts[1].trim().to_string(),
                source: parts[2].trim().to_string(),
                message: parts[3].trim().to_string(),
            })
        } else {
            None
        }
    }

    fn add_entry(&mut self, entry: LogEntry) {
        self.entries.push(entry.clone());
        self.stats.total_entries += 1;

        *self.stats.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        *self.stats.source_counts.entry(entry.source.clone()).or_insert(0) += 1;

        match entry.level.as_str() {
            "ERROR" => self.stats.errors.push(entry.clone()),
            "WARN" => self.stats.warnings.push(entry.clone()),
            _ => {}
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_source(&self, source: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.source == source)
            .collect()
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }

    pub fn get_stats(&self) -> &LogStats {
        &self.stats
    }

    pub fn get_top_sources(&self, n: usize) -> Vec<(&String, &usize)> {
        let mut sources: Vec<_> = self.stats.source_counts.iter().collect();
        sources.sort_by(|a, b| b.1.cmp(a.1));
        sources.into_iter().take(n).collect()
    }

    pub fn get_error_rate(&self) -> f64 {
        if self.stats.total_entries == 0 {
            return 0.0;
        }
        self.stats.errors.len() as f64 / self.stats.total_entries as f64 * 100.0
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
        let line = "2023-10-01 12:00:00|ERROR|auth_service|Failed to authenticate user";
        
        let entry = analyzer.parse_log_line(line).unwrap();
        assert_eq!(entry.timestamp, "2023-10-01 12:00:00");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.source, "auth_service");
        assert_eq!(entry.message, "Failed to authenticate user");
    }

    #[test]
    fn test_empty_analyzer() {
        let analyzer = LogAnalyzer::new();
        assert_eq!(analyzer.entries.len(), 0);
        assert_eq!(analyzer.get_stats().total_entries, 0);
    }
}