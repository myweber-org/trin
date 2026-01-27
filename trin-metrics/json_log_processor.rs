use serde_json::{Value, Error as JsonError};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    ParseError(JsonError),
    InvalidStructure(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<JsonError> for LogError {
    fn from(err: JsonError) -> Self {
        LogError::ParseError(err)
    }
}

pub struct LogProcessor {
    pub total_lines: usize,
    pub valid_entries: usize,
    pub error_count: usize,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            total_lines: 0,
            valid_entries: 0,
            error_count: 0,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<Value>, LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            self.total_lines += 1;
            let line_content = line?;

            if line_content.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(&line_content) {
                Ok(json_value) => {
                    if self.validate_structure(&json_value) {
                        self.valid_entries += 1;
                        results.push(json_value);
                    } else {
                        self.error_count += 1;
                    }
                }
                Err(e) => {
                    self.error_count += 1;
                    eprintln!("Failed to parse line {}: {}", self.total_lines, e);
                }
            }
        }

        Ok(results)
    }

    fn validate_structure(&self, value: &Value) -> bool {
        value.is_object() && 
        value.get("timestamp").is_some() && 
        value.get("level").is_some() && 
        value.get("message").is_some()
    }

    pub fn statistics(&self) -> String {
        format!(
            "Processed {} lines, {} valid entries, {} errors",
            self.total_lines, self.valid_entries, self.error_count
        )
    }
}

pub fn filter_by_level(logs: &[Value], level: &str) -> Vec<Value> {
    logs.iter()
        .filter(|log| {
            log.get("level")
                .and_then(|l| l.as_str())
                .map(|l| l.eq_ignore_ascii_case(level))
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}