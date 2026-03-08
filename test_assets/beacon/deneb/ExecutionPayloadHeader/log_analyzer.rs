use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        self.update_statistics();
        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() == 3 {
            Some(LogEntry {
                timestamp: parts[0].to_string(),
                level: parts[1].to_string(),
                message: parts[2].to_string(),
            })
        } else {
            None
        }
    }

    fn update_statistics(&mut self) {
        self.level_counts.clear();
        for entry in &self.entries {
            *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
    }

    pub fn get_level_summary(&self) -> &HashMap<String, usize> {
        &self.level_counts
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let mut log_data = NamedTempFile::new().unwrap();
        writeln!(log_data, "2023-10-01T10:00:00 INFO Application started").unwrap();
        writeln!(log_data, "2023-10-01T10:01:00 ERROR Database connection failed").unwrap();
        writeln!(log_data, "2023-10-01T10:02:00 WARN High memory usage detected").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.load_from_file(log_data.path()).unwrap();

        assert_eq!(analyzer.total_entries(), 3);
        let summary = analyzer.get_level_summary();
        assert_eq!(summary.get("INFO"), Some(&1));
        assert_eq!(summary.get("ERROR"), Some(&1));
        assert_eq!(summary.get("WARN"), Some(&1));
    }
}