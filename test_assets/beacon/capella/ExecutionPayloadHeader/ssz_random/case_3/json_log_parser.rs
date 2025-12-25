use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            min_level: None,
            start_time: None,
            end_time: None,
        }
    }

    pub fn with_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
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
        let json: Value = serde_json::from_str(line)?;
        
        let timestamp_str = json["timestamp"]
            .as_str()
            .ok_or("Missing timestamp field")?;
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?.with_timezone(&Utc);

        if let Some(start) = self.start_time {
            if timestamp < start {
                return Err("Before time range".into());
            }
        }

        if let Some(end) = self.end_time {
            if timestamp > end {
                return Err("After time range".into());
            }
        }

        let level = json["level"]
            .as_str()
            .ok_or("Missing level field")?
            .to_lowercase();

        if let Some(min_level) = &self.min_level {
            let level_rank = Self::level_rank(&level);
            let min_rank = Self::level_rank(min_level);
            if level_rank < min_rank {
                return Err("Below minimum level".into());
            }
        }

        let message = json["message"]
            .as_str()
            .ok_or("Missing message field")?
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    fields.insert(key.clone(), value.clone());
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

    fn level_rank(level: &str) -> u8 {
        match level {
            "error" => 4,
            "warn" => 3,
            "info" => 2,
            "debug" => 1,
            "trace" => 0,
            _ => 0,
        }
    }
}

pub fn count_errors(entries: &[LogEntry]) -> usize {
    entries.iter()
        .filter(|e| e.level == "error")
        .count()
}

pub fn group_by_level(entries: &[LogEntry]) -> HashMap<String, Vec<&LogEntry>> {
    let mut groups = HashMap::new();
    
    for entry in entries {
        groups.entry(entry.level.clone())
            .or_insert_with(Vec::new)
            .push(entry);
    }
    
    groups
}