use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra_fields: serde_json::Value,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidFormat(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
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

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }
            
            let entry: LogEntry = serde_json::from_str(&line_content)
                .map_err(|e| ParseError::InvalidFormat(
                    format!("Line {}: {}", line_num + 1, e)
                ))?;
            
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
    use std::io::Write;
    
    #[test]
    fn test_parse_valid_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"System started","user":"admin"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Connection failed","retry_count":3}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","message":"High memory usage","usage_percent":85}"#;
        
        writeln!(temp_file, "{}", log_data).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse();
        
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].message, "Connection failed");
    }
    
    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Test1"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Test2"}
{"timestamp":"2024-01-15T10:32:00Z","level":"INFO","message":"Test3"}"#;
        
        writeln!(temp_file, "{}", log_data).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.filter_by_level("INFO");
        
        assert!(result.is_ok());
        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|e| e.level == "INFO"));
    }
}