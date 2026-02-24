
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogParser {
    filters: HashMap<String, String>,
    summary_stats: HashMap<String, usize>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: HashMap::new(),
            summary_stats: HashMap::new(),
        }
    }

    pub fn add_filter(&mut self, key: &str, value: &str) {
        self.filters.insert(key.to_string(), value.to_string());
    }

    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<Value>, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut matched_logs = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| e.to_string())?;
            
            match serde_json::from_str::<Value>(&line) {
                Ok(log_entry) => {
                    if self.matches_filters(&log_entry) {
                        matched_logs.push(log_entry.clone());
                        self.update_summary(&log_entry);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse line {}: {}", line_num + 1, e);
                }
            }
        }

        Ok(matched_logs)
    }

    fn matches_filters(&self, log_entry: &Value) -> bool {
        if self.filters.is_empty() {
            return true;
        }

        for (key, expected_value) in &self.filters {
            if let Some(actual_value) = log_entry.get(key) {
                if actual_value.as_str() != Some(expected_value) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn update_summary(&mut self, log_entry: &Value) {
        if let Some(level) = log_entry.get("level").and_then(|v| v.as_str()) {
            *self.summary_stats.entry(level.to_string()).or_insert(0) += 1;
        }

        if let Some(service) = log_entry.get("service").and_then(|v| v.as_str()) {
            *self.summary_stats.entry(format!("service_{}", service)).or_insert(0) += 1;
        }
    }

    pub fn get_summary(&self) -> &HashMap<String, usize> {
        &self.summary_stats
    }

    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }

    pub fn reset_summary(&mut self) {
        self.summary_stats.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parser_with_filters() {
        let mut parser = LogParser::new();
        parser.add_filter("level", "ERROR");
        parser.add_filter("service", "api");

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "service": "api", "message": "Failed request"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "service": "api", "message": "Request processed"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "service": "db", "message": "Connection failed"}}"#).unwrap();

        let logs = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0]["level"], "ERROR");
        assert_eq!(logs[0]["service"], "api");

        let summary = parser.get_summary();
        assert_eq!(summary.get("ERROR"), Some(&1));
        assert_eq!(summary.get("service_api"), Some(&1));
    }

    #[test]
    fn test_parser_no_filters() {
        let mut parser = LogParser::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "message": "Test"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "WARN", "message": "Warning"}}"#).unwrap();

        let logs = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(logs.len(), 2);
    }
}