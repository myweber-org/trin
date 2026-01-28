use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct LogParser {
    filters: HashMap<String, String>,
    extracted_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: HashMap::new(),
            extracted_fields: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, key: &str, value: &str) {
        self.filters.insert(key.to_string(), value.to_string());
    }

    pub fn add_extracted_field(&mut self, field: &str) {
        self.extracted_fields.push(field.to_string());
    }

    pub fn parse_file(&self, file_path: &str) -> Result<Vec<HashMap<String, Value>>, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if self.matches_filters(&json_value) {
                    let extracted = self.extract_fields(&json_value);
                    results.push(extracted);
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

    fn extract_fields(&self, json_value: &Value) -> HashMap<String, Value> {
        let mut extracted = HashMap::new();
        for field in &self.extracted_fields {
            if let Some(value) = json_value.get(field) {
                extracted.insert(field.clone(), value.clone());
            }
        }
        extracted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parser_with_filters() {
        let mut parser = LogParser::new();
        parser.add_filter("level", "ERROR");
        parser.add_extracted_field("timestamp");
        parser.add_extracted_field("message");

        let test_data = json!({
            "level": "ERROR",
            "timestamp": "2023-10-01T12:00:00Z",
            "message": "Something went wrong",
            "extra": "ignored"
        });

        assert!(parser.matches_filters(&test_data));
        
        let extracted = parser.extract_fields(&test_data);
        assert_eq!(extracted.len(), 2);
        assert!(extracted.contains_key("timestamp"));
        assert!(extracted.contains_key("message"));
    }
}