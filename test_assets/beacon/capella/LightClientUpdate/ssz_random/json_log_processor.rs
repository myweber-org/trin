use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Value>, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| e.to_string())?;
            
            match serde_json::from_str::<Value>(&line) {
                Ok(json) => {
                    if self.matches_filters(&json) {
                        results.push(json);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e);
                }
            }
        }

        Ok(results)
    }

    fn matches_filters(&self, json: &Value) -> bool {
        for (key, expected_value) in &self.filters {
            if let Some(value) = json.get(key) {
                if value.as_str() != Some(expected_value) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn count_by_field(&self, logs: &[Value], field: &str) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for log in logs {
            if let Some(value) = log.get(field) {
                let key = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => continue,
                };
                *counts.entry(key).or_insert(0) += 1;
            }
        }
        
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let mut processor = LogProcessor::new();
        processor.add_filter("level", "ERROR");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "message": "Test error", "timestamp": "2023-01-01"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "message": "Test info", "timestamp": "2023-01-01"}}"#).unwrap();
        
        let results = processor.process_file(temp_file.path()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["level"], "ERROR");
    }
}