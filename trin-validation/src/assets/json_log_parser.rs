use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
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
    min_level: LogLevel,
}

impl LogParser {
    pub fn new(file_path: &str, min_level: LogLevel) -> Self {
        LogParser {
            file_path: file_path.to_string(),
            min_level,
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)?;
            if self.should_include(&entry.level) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn should_include(&self, level: &LogLevel) -> bool {
        let priority = |l: &LogLevel| match l {
            LogLevel::ERROR => 4,
            LogLevel::WARN => 3,
            LogLevel::INFO => 2,
            LogLevel::DEBUG => 1,
            LogLevel::TRACE => 0,
        };

        priority(level) >= priority(&self.min_level)
    }

    pub fn count_by_level(&self) -> Result<std::collections::HashMap<LogLevel, usize>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let mut counts = std::collections::HashMap::new();

        for entry in entries {
            *counts.entry(entry.level).or_insert(0) += 1;
        }

        Ok(counts)
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
        let log_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"ERROR","message":"Failed to connect","component":"network"}
{"timestamp":"2023-10-01T12:01:00Z","level":"INFO","message":"Connection established","component":"network"}
{"timestamp":"2023-10-01T12:02:00Z","level":"DEBUG","message":"Processing request","component":"api"}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap(), LogLevel::INFO);
        let entries = parser.parse().unwrap();
        
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, LogLevel::ERROR);
        assert_eq!(entries[1].level, LogLevel::INFO);
    }

    #[test]
    fn test_level_filtering() {
        let parser = LogParser::new("dummy", LogLevel::WARN);
        
        assert!(parser.should_include(&LogLevel::ERROR));
        assert!(parser.should_include(&LogLevel::WARN));
        assert!(!parser.should_include(&LogLevel::INFO));
        assert!(!parser.should_include(&LogLevel::DEBUG));
    }
}