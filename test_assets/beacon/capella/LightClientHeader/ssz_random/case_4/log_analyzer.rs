
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    source: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    source_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            source_counts: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.parse_line(&line);
        }

        self.update_statistics();
        Ok(())
    }

    fn parse_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() == 4 {
            let entry = LogEntry {
                timestamp: parts[0].trim().to_string(),
                level: parts[1].trim().to_string(),
                source: parts[2].trim().to_string(),
                message: parts[3].trim().to_string(),
            };
            self.entries.push(entry);
        }
    }

    fn update_statistics(&mut self) {
        self.level_counts.clear();
        self.source_counts.clear();

        for entry in &self.entries {
            *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
            *self.source_counts.entry(entry.source.clone()).or_insert(0) += 1;
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

    pub fn get_summary(&self) -> String {
        let total_entries = self.entries.len();
        let error_count = self.level_counts.get("ERROR").unwrap_or(&0);
        let warning_count = self.level_counts.get("WARNING").unwrap_or(&0);
        let info_count = self.level_counts.get("INFO").unwrap_or(&0);

        format!(
            "Total entries: {}\nErrors: {}\nWarnings: {}\nInfo: {}",
            total_entries, error_count, warning_count, info_count
        )
    }

    pub fn get_top_sources(&self, limit: usize) -> Vec<(&String, &usize)> {
        let mut sources: Vec<_> = self.source_counts.iter().collect();
        sources.sort_by(|a, b| b.1.cmp(a.1));
        sources.truncate(limit);
        sources
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "2023-10-01 10:00:00 | INFO | server | Application started"
        )
        .unwrap();
        writeln!(
            temp_file,
            "2023-10-01 10:01:00 | ERROR | database | Connection failed"
        )
        .unwrap();
        writeln!(
            temp_file,
            "2023-10-01 10:02:00 | WARNING | server | High memory usage"
        )
        .unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.load_from_file(temp_file.path()).unwrap();

        assert_eq!(analyzer.entries.len(), 3);
        assert_eq!(analyzer.filter_by_level("ERROR").len(), 1);
        assert_eq!(analyzer.filter_by_source("server").len(), 2);

        let summary = analyzer.get_summary();
        assert!(summary.contains("Total entries: 3"));
        assert!(summary.contains("Errors: 1"));
    }
}