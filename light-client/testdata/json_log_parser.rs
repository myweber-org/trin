use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid log format on line {0}")]
    InvalidFormat(usize),
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let entry: LogEntry = serde_json::from_str(&line)
                .map_err(|e| ParseError::InvalidFormat(line_num + 1))?;
            entries.push(entry);
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, ParseError> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect();
        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_logs() {
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request processed"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"db","message":"Connection failed","metadata":{"retry_count":3}}"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, log_data.as_bytes()).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse();
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].service, "db");
    }
}