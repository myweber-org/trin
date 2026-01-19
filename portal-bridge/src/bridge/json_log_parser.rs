
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
    #[serde(flatten)]
    extra_fields: HashMap<String, serde_json::Value>,
}

struct LogParser {
    min_level: Option<String>,
    filter_fields: HashMap<String, String>,
}

impl LogParser {
    fn new() -> Self {
        LogParser {
            min_level: None,
            filter_fields: HashMap::new(),
        }
    }

    fn set_min_level(&mut self, level: &str) {
        let levels = ["trace", "debug", "info", "warn", "error"];
        if levels.contains(&level.to_lowercase().as_str()) {
            self.min_level = Some(level.to_lowercase());
        }
    }

    fn add_filter(&mut self, key: &str, value: &str) {
        self.filter_fields.insert(key.to_string(), value.to_string());
    }

    fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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
        let entry: LogEntry = serde_json::from_str(line)?;

        if let Some(min_level) = &self.min_level {
            let level_order = |level: &str| -> u8 {
                match level.to_lowercase().as_str() {
                    "trace" => 1,
                    "debug" => 2,
                    "info" => 3,
                    "warn" => 4,
                    "error" => 5,
                    _ => 0,
                }
            };

            if level_order(&entry.level) < level_order(min_level) {
                return Err("Entry level below minimum threshold".into());
            }
        }

        for (key, value) in &self.filter_fields {
            if let Some(entry_value) = entry.extra_fields.get(key) {
                if entry_value.to_string() != *value {
                    return Err("Entry does not match filter criteria".into());
                }
            } else {
                return Err("Required field not found".into());
            }
        }

        Ok(entry)
    }

    fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = format!(
            "[{}] {}: {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.level.to_uppercase(),
            entry.message
        );

        if !entry.extra_fields.is_empty() {
            output.push_str(" | ");
            for (key, value) in &entry.extra_fields {
                output.push_str(&format!("{}={} ", key, value));
            }
        }

        output.trim().to_string()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    parser.set_min_level("info");
    parser.add_filter("service", "api");

    let entries = parser.parse_file("logs/app.log")?;
    
    for entry in entries {
        println!("{}", parser.format_entry(&entry));
    }

    Ok(())
}