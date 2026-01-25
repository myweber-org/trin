use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra_fields: serde_json::Value,
}

pub struct LogProcessor {
    pub entries: Vec<LogEntry>,
    pub error_count: usize,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            error_count: 0,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => {
                    eprintln!("Error parsing line {}: {}", line_num + 1, e);
                    self.error_count += 1;
                }
            }
        }

        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    pub fn to_structured_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let mut processor = LogProcessor::new();
        
        let log_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","message":"System started","user":"admin"}
{"timestamp":"2023-10-01T12:01:00Z","level":"ERROR","message":"Connection failed","attempt":3}"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, log_data.as_bytes()).unwrap();
        
        let result = processor.process_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.entries.len(), 2);
        assert_eq!(processor.error_count, 0);
        
        let errors = processor.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Connection failed");
    }
}