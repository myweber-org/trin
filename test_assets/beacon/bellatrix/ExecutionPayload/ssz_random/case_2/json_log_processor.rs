use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub struct LogProcessor {
    pub total_lines: usize,
    pub valid_json_count: usize,
    pub error_messages: Vec<String>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            total_lines: 0,
            valid_json_count: 0,
            error_messages: Vec::new(),
        }
    }

    pub fn process_file(&mut self, path: &str) -> Result<(), LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            self.total_lines += 1;
            let line = line_result?;

            match self.parse_log_line(&line) {
                Ok(json) => {
                    self.valid_json_count += 1;
                    if let Some(msg) = self.extract_error_message(&json) {
                        self.error_messages.push(msg);
                    }
                }
                Err(e) => {
                    eprintln!("Line {}: {}", self.total_lines, e);
                }
            }
        }

        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Result<Value, LogError> {
        let value: Value = serde_json::from_str(line)?;
        
        if !value.is_object() {
            return Err(LogError::MissingField("root must be object".to_string()));
        }

        Ok(value)
    }

    fn extract_error_message(&self, json: &Value) -> Option<String> {
        json.get("error")
            .or_else(|| json.get("message"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn print_summary(&self) {
        println!("Processing Summary:");
        println!("Total lines processed: {}", self.total_lines);
        println!("Valid JSON entries: {}", self.valid_json_count);
        println!("Error messages found: {}", self.error_messages.len());
        
        if !self.error_messages.is_empty() {
            println!("\nError messages:");
            for msg in &self.error_messages {
                println!("  - {}", msg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_json_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01", "level": "ERROR", "message": "Test error"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01", "level": "INFO", "data": {{"user": "test"}}}}"#).unwrap();

        let mut processor = LogProcessor::new();
        processor.process_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(processor.total_lines, 2);
        assert_eq!(processor.valid_json_count, 2);
        assert_eq!(processor.error_messages.len(), 1);
        assert_eq!(processor.error_messages[0], "Test error");
    }

    #[test]
    fn test_invalid_json_handling() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "not valid json").unwrap();
        writeln!(temp_file, r#"{{"valid": true}}"#).unwrap();

        let mut processor = LogProcessor::new();
        let result = processor.process_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.total_lines, 2);
        assert_eq!(processor.valid_json_count, 1);
    }
}