use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub struct JsonLogProcessor;

impl JsonLogProcessor {
    pub fn parse_log_file(path: &str) -> Result<Vec<LogEntry>, LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            match Self::parse_log_line(&line_content) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("Warning: Line {}: {}", line_num + 1, e),
            }
        }

        Ok(entries)
    }

    fn parse_log_line(line: &str) -> Result<LogEntry, LogError> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let timestamp = json_value["timestamp"]
            .as_str()
            .ok_or_else(|| LogError::MissingField("timestamp".to_string()))?
            .to_string();
            
        let level = json_value["level"]
            .as_str()
            .ok_or_else(|| LogError::MissingField("level".to_string()))?
            .to_string();
            
        let message = json_value["message"]
            .as_str()
            .ok_or_else(|| LogError::MissingField("message".to_string()))?
            .to_string();

        let metadata = json_value["metadata"].as_object()
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        Ok(LogEntry {
            timestamp,
            level,
            message,
            metadata,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl LogEntry {
    pub fn is_error(&self) -> bool {
        self.level.to_lowercase() == "error"
    }
    
    pub fn contains_keyword(&self, keyword: &str) -> bool {
        self.message.to_lowercase().contains(&keyword.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "System started", "metadata": {{"user": "admin"}}}}"#
        ).unwrap();
        
        let result = JsonLogProcessor::parse_log_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "INFO");
    }

    #[test]
    fn test_missing_field() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO"}}"#
        ).unwrap();
        
        let result = JsonLogProcessor::parse_log_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 0);
    }
}