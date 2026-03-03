use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;
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

    pub fn parse_logs(&self) -> Result<Vec<Value>, LogParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            let json_value: Value = serde_json::from_str(&line_content)?;

            if !json_value.is_object() {
                return Err(LogParseError::MissingField(
                    format!("Line {}: Expected JSON object", line_num + 1)
                ));
            }

            logs.push(json_value);
        }

        Ok(logs)
    }

    pub fn filter_by_level(&self, logs: &[Value], target_level: &str) -> Vec<Value> {
        logs.iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|v| v.as_str())
                    .map(|level| level.eq_ignore_ascii_case(target_level))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "message": "System started"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "message": "Disk full"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let logs = parser.parse_logs().unwrap();
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_filter_logs_by_level() {
        let logs = vec![
            serde_json::json!({"level": "INFO", "message": "test"}),
            serde_json::json!({"level": "ERROR", "message": "error"}),
            serde_json::json!({"level": "INFO", "message": "another"}),
        ];

        let parser = JsonLogParser::new("dummy.log");
        let filtered = parser.filter_by_level(&logs, "INFO");
        assert_eq!(filtered.len(), 2);
    }
}