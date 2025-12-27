use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line, e),
            }
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level == level)
            .collect();
        Ok(filtered)
    }

    pub fn count_entries(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        Ok(entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_log_entries() {
        let json_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","component":"boot"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Disk full","component":"storage"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, LogLevel::INFO);
        assert_eq!(entries[1].level, LogLevel::ERROR);
    }

    #[test]
    fn test_filter_error_logs() {
        let json_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","component":"boot"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Disk full","component":"storage"}
{"timestamp":"2024-01-15T10:32:00Z","level":"ERROR","message":"Network timeout","component":"network"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level(LogLevel::ERROR).unwrap();

        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|entry| entry.level == LogLevel::ERROR));
    }
}