use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct LogParser {
    filters: HashMap<String, String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: HashMap::new(),
        }
    }

    pub fn add_filter(&mut self, key: &str, value: &str) {
        self.filters.insert(key.to_string(), value.to_string());
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if self.matches_filters(&json_value) {
                    results.push(json_value);
                }
            }
        }

        Ok(results)
    }

    fn matches_filters(&self, json_value: &Value) -> bool {
        for (key, expected_value) in &self.filters {
            if let Some(actual_value) = json_value.get(key) {
                if actual_value.as_str() != Some(expected_value) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn extract_field(&self, json_value: &Value, field_path: &str) -> Option<&Value> {
        let mut current = json_value;
        for part in field_path.split('.') {
            current = current.get(part)?;
        }
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matching() {
        let mut parser = LogParser::new();
        parser.add_filter("level", "ERROR");
        
        let log_entry = serde_json::json!({
            "timestamp": "2023-10-01T12:00:00Z",
            "level": "ERROR",
            "message": "Something went wrong"
        });

        assert!(parser.matches_filters(&log_entry));
    }

    #[test]
    fn test_field_extraction() {
        let parser = LogParser::new();
        let data = serde_json::json!({
            "user": {
                "name": "alice",
                "id": 42
            }
        });

        let extracted = parser.extract_field(&data, "user.name");
        assert_eq!(extracted, Some(&serde_json::json!("alice")));
    }
}