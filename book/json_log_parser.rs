use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error at line {line}: {source}")]
    JsonParse {
        line: usize,
        source: serde_json::Error,
    },
    #[error("Missing required field '{field}' at line {line}")]
    MissingField { line: usize, field: String },
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_json_log_file(path: &str) -> Result<Vec<LogEntry>, LogParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let line_number = line_num + 1;

        let json_value: Value = serde_json::from_str(&line)
            .map_err(|e| LogParseError::JsonParse {
                line: line_number,
                source: e,
            })?;

        let obj = json_value.as_object().ok_or_else(|| {
            LogParseError::MissingField {
                line: line_number,
                field: "object".to_string(),
            }
        })?;

        let timestamp = obj
            .get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LogParseError::MissingField {
                line: line_number,
                field: "timestamp".to_string(),
            })?
            .to_string();

        let level = obj
            .get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LogParseError::MissingField {
                line: line_number,
                field: "level".to_string(),
            })?
            .to_string();

        let message = obj
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LogParseError::MissingField {
                line: line_number,
                field: "message".to_string(),
            })?
            .to_string();

        let metadata = obj
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        entries.push(LogEntry {
            timestamp,
            level,
            message,
            metadata,
        });
    }

    Ok(entries)
}

pub fn filter_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level.eq_ignore_ascii_case(level))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Service started","metadata":{"pid":1234}}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Connection failed","metadata":{"retry_count":3}}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let entries = parse_json_log_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].level, "ERROR");
    }

    #[test]
    fn test_filter_by_level() {
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Test info".to_string(),
                metadata: json!({}),
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Test error".to_string(),
                metadata: json!({}),
            },
            LogEntry {
                timestamp: "2024-01-15T10:32:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Another info".to_string(),
                metadata: json!({}),
            },
        ];

        let errors = filter_by_level(&entries, "ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Test error");
    }
}