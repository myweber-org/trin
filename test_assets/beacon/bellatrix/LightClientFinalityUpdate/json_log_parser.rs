use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;

pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    fields: HashMap<String, Value>,
}

impl LogEntry {
    pub fn new(timestamp: String, level: String, message: String, fields: HashMap<String, Value>) -> Self {
        LogEntry {
            timestamp,
            level,
            message,
            fields,
        }
    }

    pub fn matches_filter(&self, filter: &LogFilter) -> bool {
        if let Some(level_filter) = &filter.level {
            if !level_filter.contains(&self.level) {
                return false;
            }
        }

        if let Some(message_filter) = &filter.message_contains {
            if !self.message.contains(message_filter) {
                return false;
            }
        }

        if let Some(field_filters) = &filter.field_filters {
            for (key, expected_value) in field_filters {
                match self.fields.get(key) {
                    Some(actual_value) => {
                        if actual_value != expected_value {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
        }

        true
    }
}

pub struct LogFilter {
    level: Option<Vec<String>>,
    message_contains: Option<String>,
    field_filters: Option<HashMap<String, Value>>,
}

impl LogFilter {
    pub fn new() -> Self {
        LogFilter {
            level: None,
            message_contains: None,
            field_filters: None,
        }
    }

    pub fn with_level(mut self, levels: Vec<&str>) -> Self {
        self.level = Some(levels.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn with_message_contains(mut self, text: &str) -> Self {
        self.message_contains = Some(text.to_string());
        self
    }

    pub fn with_field_filter(mut self, key: &str, value: Value) -> Self {
        let mut filters = self.field_filters.unwrap_or_default();
        filters.insert(key.to_string(), value);
        self.field_filters = Some(filters);
        self
    }
}

pub struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser { entries: Vec::new() }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                self.entries.push(entry);
                count += 1;
            }
        }

        Ok(count)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json: Value = serde_json::from_str(line)?;

        let timestamp = json["timestamp"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let level = json["level"]
            .as_str()
            .unwrap_or("INFO")
            .to_string();

        let message = json["message"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }

        Ok(LogEntry::new(timestamp, level, message, fields))
    }

    pub fn filter(&self, filter: &LogFilter) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.matches_filter(filter))
            .collect()
    }

    pub fn summarize(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for entry in &self.entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }
        summary
    }

    pub fn get_entries_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_log_parsing() {
        let mut parser = LogParser::new();
        let test_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","service":"api","error_code":500}"#;
        
        let entry = parser.parse_line(test_data).unwrap();
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.fields.get("service").unwrap(), &json!("api"));
    }

    #[test]
    fn test_filtering() {
        let mut parser = LogParser::new();
        parser.entries.push(LogEntry::new(
            "2024-01-15T10:30:00Z".to_string(),
            "ERROR".to_string(),
            "Database connection failed".to_string(),
            HashMap::new(),
        ));
        
        parser.entries.push(LogEntry::new(
            "2024-01-15T10:31:00Z".to_string(),
            "INFO".to_string(),
            "Request processed".to_string(),
            HashMap::new(),
        ));

        let filter = LogFilter::new().with_level(vec!["ERROR"]);
        let filtered = parser.filter(&filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].level, "ERROR");
    }
}