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

#[derive(Debug)]
pub struct LogStats {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    unique_messages: HashMap<String, usize>,
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
                error_count: 0,
                warning_count: 0,
                info_count: 0,
                unique_messages: HashMap::new(),
            },
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

            self.update_stats(&entry);
            self.entries.push(entry);
        }
    }

    fn update_stats(&mut self, entry: &LogEntry) {
        self.stats.total_entries += 1;

        match entry.level.as_str() {
            "ERROR" => self.stats.error_count += 1,
            "WARNING" => self.stats.warning_count += 1,
            "INFO" => self.stats.info_count += 1,
            _ => {}
        }

        *self.stats.unique_messages.entry(entry.message.clone()).or_insert(0) += 1;
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_stats(&self) -> &LogStats {
        &self.stats
    }

    pub fn find_duplicate_messages(&self) -> Vec<(&String, &usize)> {
        self.stats
            .unique_messages
            .iter()
            .filter(|(_, &count)| count > 1)
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01T10:00:00 INFO System started").unwrap();
        writeln!(temp_file, "2023-10-01T10:01:00 ERROR Database connection failed").unwrap();
        writeln!(temp_file, "2023-10-01T10:02:00 WARNING High memory usage").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        let stats = analyzer.get_stats();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.warning_count, 1);
        assert_eq!(stats.info_count, 1);
        
        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Database connection failed");
    }
}