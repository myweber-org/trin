use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: String,
    field_filter: Option<HashMap<String, Value>>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            field_filter: None,
        }
    }

    pub fn with_field_filter(mut self, filter: HashMap<String, Value>) -> Self {
        self.field_filter = Some(filter);
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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

    pub fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let timestamp = json_value.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let level = json_value.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_lowercase();

        if !self.is_level_allowed(&level) {
            return Err("Log level below minimum threshold".into());
        }

        let message = json_value.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }

        if let Some(filter) = &self.field_filter {
            for (key, value) in filter {
                if fields.get(key) != Some(value) {
                    return Err("Entry does not match field filter".into());
                }
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn is_level_allowed(&self, level: &str) -> bool {
        let level_order = vec!["trace", "debug", "info", "warn", "error", "fatal"];
        
        let min_index = level_order.iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(2);
        
        let entry_index = level_order.iter()
            .position(|&l| l == level)
            .unwrap_or(2);

        entry_index >= min_index
    }
}

pub fn format_entries(entries: &[LogEntry], show_fields: bool) -> String {
    let mut output = String::new();
    
    for entry in entries {
        output.push_str(&format!(
            "[{}] {}: {}\n",
            entry.timestamp,
            entry.level.to_uppercase(),
            entry.message
        ));

        if show_fields && !entry.fields.is_empty() {
            for (key, value) in &entry.fields {
                output.push_str(&format!("  {}: {}\n", key, value));
            }
            output.push('\n');
        }
    }

    output
}