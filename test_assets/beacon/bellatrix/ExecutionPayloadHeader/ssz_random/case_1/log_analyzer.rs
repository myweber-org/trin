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
            self.parse_line(&line);
        }

        Ok(())
    }

    fn parse_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() == 3 {
            let entry = LogEntry {
                timestamp: parts[0].to_string(),
                level: parts[1].to_string(),
                message: parts[2].to_string(),
            };

            *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
            self.entries.push(entry);
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_statistics(&self) -> &HashMap<String, usize> {
        &self.level_counts
    }

    pub fn count_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn find_messages_containing(&self, keyword: &str) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .map(|entry| entry.message.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01T10:00:00 INFO Application started").unwrap();
        writeln!(temp_file, "2023-10-01T10:01:00 ERROR Database connection failed").unwrap();
        writeln!(temp_file, "2023-10-01T10:02:00 WARN High memory usage detected").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.count_entries(), 3);
        
        let error_logs = analyzer.filter_by_level("ERROR");
        assert_eq!(error_logs.len(), 1);
        assert_eq!(error_logs[0].message, "Database connection failed");
        
        let stats = analyzer.get_statistics();
        assert_eq!(stats.get("INFO"), Some(&1));
        assert_eq!(stats.get("ERROR"), Some(&1));
        assert_eq!(stats.get("WARN"), Some(&1));
    }
}