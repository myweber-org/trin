use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogFilter {
    min_level: Option<String>,
    contains_text: Option<String>,
}

impl LogFilter {
    pub fn new(min_level: Option<String>, contains_text: Option<String>) -> Self {
        LogFilter {
            min_level,
            contains_text,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(parsed) = serde_json::from_str::<Value>(&line) {
                if self.matches_filter(&parsed) {
                    results.push(line);
                }
            }
        }

        Ok(results)
    }

    fn matches_filter(&self, log_entry: &Value) -> bool {
        if let Some(min_level) = &self.min_level {
            if let Some(level) = log_entry.get("level").and_then(|v| v.as_str()) {
                let level_order = Self::level_order(level);
                let min_order = Self::level_order(min_level);
                if level_order < min_order {
                    return false;
                }
            }
        }

        if let Some(text) = &self.contains_text {
            let log_string = log_entry.to_string();
            if !log_string.contains(text) {
                return false;
            }
        }

        true
    }

    fn level_order(level: &str) -> u8 {
        match level.to_lowercase().as_str() {
            "debug" => 1,
            "info" => 2,
            "warn" => 3,
            "error" => 4,
            "critical" => 5,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_by_level() {
        let logs = r#"{"level": "debug", "message": "test debug"}
{"level": "error", "message": "test error"}
{"level": "info", "message": "test info"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", logs).unwrap();

        let filter = LogFilter::new(Some("info".to_string()), None);
        let results = filter.process_file(temp_file.path()).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].contains("error"));
        assert!(results[1].contains("info"));
    }

    #[test]
    fn test_filter_by_text() {
        let logs = r#"{"level": "info", "message": "user login"}
{"level": "error", "message": "database failure"}
{"level": "info", "message": "user logout"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", logs).unwrap();

        let filter = LogFilter::new(None, Some("user".to_string()));
        let results = filter.process_file(temp_file.path()).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].contains("login"));
        assert!(results[1].contains("logout"));
    }
}