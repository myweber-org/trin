use serde_json::{Value, Map};
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
    filter_level: Option<String>,
    required_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filter_level: None,
            required_fields: Vec::new(),
        }
    }

    pub fn with_level_filter(mut self, level: &str) -> Self {
        self.filter_level = Some(level.to_uppercase());
        self
    }

    pub fn with_required_fields(mut self, fields: &[&str]) -> Self {
        self.required_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, String> {
        let json_value: Value = serde_json::from_str(line)
            .map_err(|e| format!("Invalid JSON: {}", e))?;
        
        let obj = json_value.as_object()
            .ok_or_else(|| "Expected JSON object".to_string())?;
        
        let timestamp = obj.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing timestamp field".to_string())?
            .to_string();
        
        let level = obj.get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing level field".to_string())?
            .to_uppercase();
        
        if let Some(filter) = &self.filter_level {
            if &level != filter {
                return Err("Level filtered out".to_string());
            }
        }
        
        let message = obj.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let mut fields = HashMap::new();
        for (key, value) in obj {
            if !["timestamp", "level", "message"].contains(&key.as_str()) {
                fields.insert(key.clone(), value.clone());
            }
        }
        
        for required in &self.required_fields {
            if !fields.contains_key(required) {
                return Err(format!("Missing required field: {}", required));
            }
        }
        
        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    pub fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = format!("[{}] {}: {}", entry.timestamp, entry.level, entry.message);
        
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
    let mut stats = HashMap::new();
    
    for entry in entries {
        *stats.entry(entry.level.clone()).or_insert(0) += 1;
    }
    
    stats
}