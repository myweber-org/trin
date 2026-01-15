
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<usize, String> {
        if records.is_empty() {
            return Err("No valid records found".to_string());
        }

        let expected_len = records[0].len();
        for (i, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!(
                    "Record {} has {} fields, expected {}",
                    i + 1,
                    record.len(),
                    expected_len
                ));
            }
        }

        Ok(records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_validation() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let result = processor.validate_records(&records);
        
        assert_eq!(result, Ok(2));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    ValueOutOfRange(f64),
    #[error("Duplicate record ID: {0}")]
    DuplicateId(u64),
}

pub struct DataProcessor {
    validation_enabled: bool,
    max_value_limit: Option<f64>,
    seen_ids: std::collections::HashSet<u64>,
}

impl DataProcessor {
    pub fn new(validation_enabled: bool, max_value_limit: Option<f64>) -> Self {
        Self {
            validation_enabled,
            max_value_limit,
            seen_ids: std::collections::HashSet::new(),
        }
    }

    pub fn process_record(&mut self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        if self.validation_enabled {
            self.validate_record(record)?;
        }
        
        let transformed = self.transform_record(record);
        Ok(transformed)
    }

    fn validate_record(&mut self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        if record.values.is_empty() {
            return Err(ProcessingError::MissingField("values".to_string()));
        }

        if self.seen_ids.contains(&record.id) {
            return Err(ProcessingError::DuplicateId(record.id));
        }

        if let Some(limit) = self.max_value_limit {
            for &value in record.values.values() {
                if value > limit {
                    return Err(ProcessingError::ValueOutOfRange(value));
                }
            }
        }

        self.seen_ids.insert(record.id);
        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> DataRecord {
        let mut transformed_values = HashMap::new();
        
        for (key, value) in &record.values {
            let transformed_key = key.to_lowercase().replace(' ', "_");
            let transformed_value = if *value < 0.0 {
                0.0
            } else {
                *value
            };
            transformed_values.insert(transformed_key, transformed_value);
        }

        let mut transformed_tags = record.tags.clone();
        transformed_tags.sort();
        transformed_tags.dedup();

        DataRecord {
            id: record.id,
            timestamp: record.timestamp,
            values: transformed_values,
            tags: transformed_tags,
        }
    }

    pub fn reset_processor(&mut self) {
        self.seen_ids.clear();
    }

    pub fn get_processed_count(&self) -> usize {
        self.seen_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let mut processor = DataProcessor::new(true, Some(100.0));
        
        let mut values = HashMap::new();
        values.insert("Temperature".to_string(), 25.5);
        values.insert("Pressure".to_string(), 101.3);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            tags: vec!["sensor".to_string(), "room1".to_string()],
        };

        let result = processor.process_record(&record);
        assert!(result.is_ok());
        assert_eq!(processor.get_processed_count(), 1);
    }

    #[test]
    fn test_invalid_timestamp() {
        let mut processor = DataProcessor::new(true, None);
        
        let record = DataRecord {
            id: 1,
            timestamp: -1,
            values: HashMap::from([("test".to_string(), 1.0)]),
            tags: vec![],
        };

        let result = processor.process_record(&record);
        assert!(matches!(result, Err(ProcessingError::InvalidTimestamp(-1))));
    }

    #[test]
    fn test_value_limit_exceeded() {
        let mut processor = DataProcessor::new(true, Some(50.0));
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: HashMap::from([("reading".to_string(), 75.0)]),
            tags: vec![],
        };

        let result = processor.process_record(&record);
        assert!(matches!(result, Err(ProcessingError::ValueOutOfRange(75.0))));
    }
}