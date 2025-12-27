use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
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

pub struct JsonLogParser {
    file_path: String,
    required_fields: Vec<String>,
}

impl JsonLogParser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            required_fields: vec!["timestamp".to_string(), "level".to_string()],
        }
    }

    pub fn with_required_fields(mut self, fields: Vec<&str>) -> Self {
        self.required_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn parse(&self) -> Result<Vec<Value>, LogParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut parsed_logs = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let line_number = line_num + 1;

            if line.trim().is_empty() {
                continue;
            }

            let json_value: Value = serde_json::from_str(&line)
                .map_err(|e| LogParseError::JsonParse {
                    line: line_number,
                    source: e,
                })?;

            self.validate_fields(&json_value, line_number)?;
            parsed_logs.push(json_value);
        }

        Ok(parsed_logs)
    }

    fn validate_fields(&self, json_value: &Value, line_number: usize) -> Result<(), LogParseError> {
        if let Value::Object(map) = json_value {
            for field in &self.required_fields {
                if !map.contains_key(field) {
                    return Err(LogParseError::MissingField {
                        line: line_number,
                        field: field.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn filter_by_level(logs: &[Value], level: &str) -> Vec<&Value> {
        logs.iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|v| v.as_str())
                    .map(|l| l.eq_ignore_ascii_case(level))
                    .unwrap_or(false)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parser_with_valid_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let logs = vec![
            json!({"timestamp": "2024-01-01T00:00:00Z", "level": "INFO", "message": "System started"}),
            json!({"timestamp": "2024-01-01T00:01:00Z", "level": "ERROR", "message": "Disk full", "code": 1001}),
        ];

        for log in &logs {
            writeln!(temp_file, "{}", log).unwrap();
        }

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let parsed = parser.parse().unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["level"], "INFO");
        assert_eq!(parsed[1]["code"], 1001);
    }

    #[test]
    fn test_filter_by_level() {
        let logs = vec![
            json!({"timestamp": "2024-01-01T00:00:00Z", "level": "INFO", "message": "Test"}),
            json!({"timestamp": "2024-01-01T00:01:00Z", "level": "ERROR", "message": "Error"}),
            json!({"timestamp": "2024-01-01T00:02:00Z", "level": "INFO", "message": "Another"}),
        ];

        let info_logs: Vec<Value> = logs.clone();
        let filtered = JsonLogParser::filter_by_level(&info_logs, "INFO");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_missing_required_field() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{{\"level\": \"INFO\", \"message\": \"No timestamp\"}}").unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse();
        assert!(matches!(result, Err(LogParseError::MissingField { line: 1, field: _ })));
    }
}