use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidFormat(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line_content)
                .map_err(|e| ParseError::InvalidFormat(format!("Line {}: {}", line_num + 1, e)))?;
            
            entries.push(entry);
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, ParseError> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect();
        
        Ok(filtered)
    }

    pub fn count_entries(&self) -> Result<usize, ParseError> {
        let entries = self.parse()?;
        Ok(entries.len())
    }
}

pub fn analyze_logs(file_path: &str) -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new(file_path);
    
    println!("Analyzing logs from: {}", file_path);
    
    let total_entries = parser.count_entries()?;
    println!("Total log entries: {}", total_entries);
    
    let error_logs = parser.filter_by_level("error")?;
    println!("Error entries: {}", error_logs.len());
    
    let warning_logs = parser.filter_by_level("warning")?;
    println!("Warning entries: {}", warning_logs.len());
    
    if !error_logs.is_empty() {
        println!("\nRecent errors:");
        for entry in error_logs.iter().take(5) {
            println!("[{}] {}: {}", entry.timestamp, entry.service, entry.message);
        }
    }
    
    Ok(())
}use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Option<DateTime<FixedOffset>>,
    pub level: Option<String>,
    pub message: Option<String>,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    filters: Vec<Filter>,
    format_options: FormatOptions,
}

#[derive(Clone)]
pub enum Filter {
    Level(String),
    FieldEquals(String, Value),
    FieldExists(String),
    Custom(Box<dyn Fn(&LogEntry) -> bool + Send + Sync>),
}

pub struct FormatOptions {
    pub show_timestamp: bool,
    pub show_level: bool,
    pub show_fields: bool,
    pub indent: usize,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: Vec::new(),
            format_options: FormatOptions {
                show_timestamp: true,
                show_level: true,
                show_fields: false,
                indent: 2,
            },
        }
    }

    pub fn add_filter(&mut self, filter: Filter) -> &mut Self {
        self.filters.push(filter);
        self
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                if self.passes_filters(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json: Value = serde_json::from_str(line)?;
        let mut entry = LogEntry {
            timestamp: None,
            level: None,
            message: None,
            fields: HashMap::new(),
        };

        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                match key.as_str() {
                    "timestamp" => {
                        if let Some(ts_str) = value.as_str() {
                            if let Ok(dt) = DateTime::parse_from_rfc3339(ts_str) {
                                entry.timestamp = Some(dt);
                            }
                        }
                    }
                    "level" => {
                        entry.level = value.as_str().map(|s| s.to_string());
                    }
                    "message" => {
                        entry.message = value.as_str().map(|s| s.to_string());
                    }
                    _ => {
                        entry.fields.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        Ok(entry)
    }

    fn passes_filters(&self, entry: &LogEntry) -> bool {
        self.filters.iter().all(|filter| match filter {
            Filter::Level(level) => entry.level.as_ref().map_or(false, |l| l == level),
            Filter::FieldEquals(key, expected) => {
                entry.fields.get(key).map_or(false, |v| v == expected)
            }
            Filter::FieldExists(key) => entry.fields.contains_key(key),
            Filter::Custom(func) => func(entry),
        })
    }

    pub fn format_entry(&self, entry: &LogEntry) -> String {
        let mut parts = Vec::new();

        if self.format_options.show_timestamp {
            if let Some(ts) = &entry.timestamp {
                parts.push(format!("[{}]", ts.format("%Y-%m-%d %H:%M:%S")));
            }
        }

        if self.format_options.show_level {
            if let Some(level) = &entry.level {
                parts.push(format!("{}:", level.to_uppercase()));
            }
        }

        if let Some(msg) = &entry.message {
            parts.push(msg.clone());
        }

        if self.format_options.show_fields && !entry.fields.is_empty() {
            let fields_str = entry.fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("{{{}}}", fields_str));
        }

        parts.join(" ")
    }

    pub fn set_format_options(&mut self, options: FormatOptions) -> &mut Self {
        self.format_options = options;
        self
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_json_log() {
        let parser = LogParser::new();
        let log_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"Service started","service":"api","port":8080}"#;
        
        let entry = parser.parse_line(log_line).unwrap();
        assert_eq!(entry.level, Some("info".to_string()));
        assert_eq!(entry.message, Some("Service started".to_string()));
        assert_eq!(entry.fields.get("service"), Some(&json!("api")));
        assert_eq!(entry.fields.get("port"), Some(&json!(8080)));
    }

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        parser.add_filter(Filter::Level("error".to_string()));

        let entry = LogEntry {
            timestamp: None,
            level: Some("info".to_string()),
            message: Some("test".to_string()),
            fields: HashMap::new(),
        };

        assert!(!parser.passes_filters(&entry));
    }
}