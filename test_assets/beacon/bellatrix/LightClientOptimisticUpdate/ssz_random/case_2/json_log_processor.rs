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
pub enum LogError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    InvalidFormat(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<serde_json::Error> for LogError {
    fn from(err: serde_json::Error) -> Self {
        LogError::ParseError(err)
    }
}

pub struct LogProcessor {
    entries: Vec<LogEntry>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line)
                .map_err(|e| LogError::InvalidFormat(format!("Line {}: {}", line_num + 1, e)))?;
            
            self.entries.push(entry);
        }

        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    pub fn get_service_stats(&self) -> std::collections::HashMap<String, usize> {
        let mut stats = std::collections::HashMap::new();
        
        for entry in &self.entries {
            *stats.entry(entry.service.clone()).or_insert(0) += 1;
        }
        
        stats
    }

    pub fn export_to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &self.entries)?;
        Ok(())
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let mut processor = LogProcessor::new();
        
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "service": "api", "message": "Request processed"}
{"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "service": "db", "message": "Connection failed", "metadata": {"retry_count": 3}}"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), log_data).unwrap();
        
        assert!(processor.load_from_file(temp_file.path()).is_ok());
        assert_eq!(processor.entries().len(), 2);
        
        let errors = processor.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        
        let stats = processor.get_service_stats();
        assert_eq!(stats.get("api"), Some(&1));
        assert_eq!(stats.get("db"), Some(&1));
    }
}