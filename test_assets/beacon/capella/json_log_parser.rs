use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub raw_data: Value,
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

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                let timestamp_str = json_value["timestamp"].as_str().unwrap_or("");
                let level = json_value["level"].as_str().unwrap_or("INFO").to_string();
                let message = json_value["message"].as_str().unwrap_or("").to_string();

                if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp_str) {
                    entries.push(LogEntry {
                        timestamp: timestamp.with_timezone(&Utc),
                        level,
                        message,
                        raw_data: json_value,
                    });
                }
            }
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level.to_uppercase() == level.to_uppercase())
            .collect();
        Ok(filtered)
    }

    pub fn filter_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect();
        Ok(filtered)
    }

    pub fn count_entries(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        Ok(entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "ERROR", "message": "Database connection failed"}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"timestamp": "2024-01-15T11:45:00Z", "level": "INFO", "message": "Server started successfully"}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"timestamp": "2024-01-15T12:15:00Z", "level": "WARN", "message": "High memory usage detected"}}"#
        )
        .unwrap();
        file
    }

    #[test]
    fn test_parse_logs() {
        let test_file = create_test_log();
        let parser = LogParser::new(test_file.path().to_str().unwrap());
        let result = parser.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_filter_by_level() {
        let test_file = create_test_log();
        let parser = LogParser::new(test_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Database connection failed");
    }
}