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
    field_filters: HashMap<String, String>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            field_filters: HashMap::new(),
        }
    }

    pub fn add_field_filter(&mut self, key: &str, value: &str) {
        self.field_filters.insert(key.to_string(), value.to_string());
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

        let timestamp = json_value["timestamp"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let level = json_value["level"]
            .as_str()
            .unwrap_or("info")
            .to_lowercase();

        if !self.meets_level_requirement(&level) {
            return Err("Log level below minimum threshold".into());
        }

        let message = json_value["message"]
            .as_str()
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

        if !self.passes_field_filters(&fields) {
            return Err("Log entry does not match field filters".into());
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn meets_level_requirement(&self, level: &str) -> bool {
        let level_order = vec!["trace", "debug", "info", "warn", "error", "fatal"];
        
        let min_index = level_order.iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(2);
        
        let entry_index = level_order.iter()
            .position(|&l| l == level)
            .unwrap_or(2);

        entry_index >= min_index
    }

    fn passes_field_filters(&self, fields: &HashMap<String, Value>) -> bool {
        for (filter_key, filter_value) in &self.field_filters {
            match fields.get(filter_key) {
                Some(value) => {
                    if let Some(str_value) = value.as_str() {
                        if str_value != filter_value {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }

    pub fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = format!(
            "[{}] {}: {}",
            entry.timestamp,
            entry.level.to_uppercase(),
            entry.message
        );

        if !entry.fields.is_empty() {
            output.push_str(" | ");
            let fields_str: Vec<String> = entry.fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            output.push_str(&fields_str.join(", "));
        }

        output
    }
}

pub fn analyze_logs(entries: &[LogEntry]) -> HashMap<String, usize> {
    let mut analysis = HashMap::new();
    
    for entry in entries {
        *analysis.entry(entry.level.clone()).or_insert(0) += 1;
    }
    
    analysis
}