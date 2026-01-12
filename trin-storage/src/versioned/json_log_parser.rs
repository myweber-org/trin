use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};

pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    message: String,
    fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: Option<String>,
    field_filters: HashMap<String, Value>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            min_level: None,
            field_filters: HashMap::new(),
        }
    }

    pub fn set_min_level(&mut self, level: &str) {
        self.min_level = Some(level.to_lowercase());
    }

    pub fn add_field_filter(&mut self, key: &str, value: Value) {
        self.field_filters.insert(key.to_string(), value);
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let timestamp_str = json_value["timestamp"]
            .as_str()
            .ok_or("Missing timestamp field")?;
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?;

        let level = json_value["level"]
            .as_str()
            .ok_or("Missing level field")?
            .to_string();

        if let Some(min_level) = &self.min_level {
            if !self.is_level_allowed(&level, min_level) {
                return Err("Log level below minimum threshold".into());
            }
        }

        let message = json_value["message"]
            .as_str()
            .ok_or("Missing message field")?
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }

        for (filter_key, filter_value) in &self.field_filters {
            if let Some(actual_value) = fields.get(filter_key) {
                if actual_value != filter_value {
                    return Err("Field filter mismatch".into());
                }
            } else {
                return Err("Required field not found".into());
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn is_level_allowed(&self, log_level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error"];
        let log_idx = levels.iter().position(|&l| l == log_level.to_lowercase());
        let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());

        match (log_idx, min_idx) {
            (Some(l), Some(m)) => l >= m,
            _ => false,
        }
    }
}

impl LogEntry {
    pub fn format(&self, include_fields: bool) -> String {
        let mut output = format!(
            "[{}] {}: {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.level.to_uppercase(),
            self.message
        );

        if include_fields && !self.fields.is_empty() {
            output.push_str(" | Fields: ");
            let field_strings: Vec<String> = self.fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            output.push_str(&field_strings.join(", "));
        }

        output
    }
}