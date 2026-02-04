use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct LogProcessor {
    filters: HashMap<String, String>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            filters: HashMap::new(),
        }
    }

    pub fn add_filter(&mut self, key: &str, value: &str) {
        self.filters.insert(key.to_string(), value.to_string());
    }

    pub fn process_file(&self, path: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut matched_logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if self.matches_filters(&json_value) {
                    matched_logs.push(json_value);
                }
            }
        }

        Ok(matched_logs)
    }

    fn matches_filters(&self, json: &Value) -> bool {
        for (key, expected_value) in &self.filters {
            if let Some(actual_value) = json.get(key) {
                if actual_value.as_str() != Some(expected_value) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_filter_matching() {
        let mut processor = LogProcessor::new();
        processor.add_filter("level", "ERROR");
        processor.add_filter("service", "api");

        let log_entry = json!({
            "timestamp": "2024-01-15T10:30:00Z",
            "level": "ERROR",
            "service": "api",
            "message": "Database connection failed"
        });

        assert!(processor.matches_filters(&log_entry));
    }

    #[test]
    fn test_filter_non_matching() {
        let mut processor = LogProcessor::new();
        processor.add_filter("level", "ERROR");

        let log_entry = json!({
            "level": "INFO",
            "message": "Service started"
        });

        assert!(!processor.matches_filters(&log_entry));
    }
}