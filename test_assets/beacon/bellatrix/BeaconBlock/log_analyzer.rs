use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

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

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    pub fn count_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn get_errors(&self) -> Vec<&LogEntry> {
        self.filter_by_level("ERROR")
    }

    pub fn get_warnings(&self) -> Vec<&LogEntry> {
        self.filter_by_level("WARN")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut analyzer = LogAnalyzer::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "2023-10-01T10:30:00 INFO Application started").unwrap();
        writeln!(temp_file, "2023-10-01T10:31:00 ERROR Database connection failed").unwrap();
        writeln!(temp_file, "2023-10-01T10:32:00 WARN High memory usage detected").unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.count_entries(), 3);
        assert_eq!(analyzer.get_errors().len(), 1);
        assert_eq!(analyzer.get_warnings().len(), 1);
    }
}