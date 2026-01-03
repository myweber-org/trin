use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub struct JsonLogParser {
    file_path: String,
}

impl JsonLogParser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<Value>, LogParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let json_value: Value = serde_json::from_str(&line)?;
            logs.push(json_value);
        }

        Ok(logs)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<Value>, LogParseError> {
        let logs = self.parse()?;
        let filtered: Vec<Value> = logs
            .into_iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|v| v.as_str())
                    .map(|l| l.eq_ignore_ascii_case(level))
                    .unwrap_or(false)
            })
            .collect();

        Ok(filtered)
    }

    pub fn extract_timestamps(&self) -> Result<Vec<String>, LogParseError> {
        let logs = self.parse()?;
        let mut timestamps = Vec::new();

        for log in logs {
            if let Some(Value::String(ts)) = log.get("timestamp") {
                timestamps.push(ts.clone());
            }
        }

        Ok(timestamps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "Service started"}
{"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "message": "Connection failed"}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"level": "INFO", "msg": "test1"}
{"level": "ERROR", "msg": "test2"}
{"level": "INFO", "msg": "test3"}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
    }
}