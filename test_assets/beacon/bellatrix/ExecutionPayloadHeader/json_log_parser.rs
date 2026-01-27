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
    extra: HashMap<String, serde_json::Value>,
}

struct LogParser {
    min_level: String,
    search_term: Option<String>,
}

impl LogParser {
    fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            search_term: None,
        }
    }

    fn with_search(mut self, term: &str) -> Self {
        self.search_term = Some(term.to_lowercase());
        self
    }

    fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.should_include(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        let level_ok = match entry.level.to_lowercase().as_str() {
            "error" => self.min_level == "error",
            "warn" => self.min_level == "error" || self.min_level == "warn",
            "info" => self.min_level != "debug",
            "debug" => true,
            _ => false,
        };

        if !level_ok {
            return false;
        }

        if let Some(ref term) = self.search_term {
            entry.message.to_lowercase().contains(term) ||
            entry.extra.values().any(|v| 
                v.as_str()
                 .map(|s| s.to_lowercase().contains(term))
                 .unwrap_or(false)
            )
        } else {
            true
        }
    }

    fn format_entry(&self, entry: &LogEntry) -> String {
        let mut output = format!(
            "[{}] {}: {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.level.to_uppercase(),
            entry.message
        );

        if !entry.extra.is_empty() {
            output.push_str(" | ");
            for (key, value) in &entry.extra {
                output.push_str(&format!("{}={:?} ", key, value));
            }
        }

        output
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = LogParser::new("info")
        .with_search("connection");

    let entries = parser.parse_file("application.log")?;
    
    for entry in entries {
        println!("{}", parser.format_entry(&entry));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_parser_filtering() {
        let mut extra = HashMap::new();
        extra.insert("user_id".to_string(), serde_json::json!("user123"));
        
        let entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2023, 10, 5, 14, 30, 0).unwrap(),
            level: "info".to_string(),
            message: "User connection established".to_string(),
            extra,
        };

        let parser = LogParser::new("info").with_search("connection");
        assert!(parser.should_include(&entry));

        let parser = LogParser::new("error");
        assert!(!parser.should_include(&entry));
    }
}