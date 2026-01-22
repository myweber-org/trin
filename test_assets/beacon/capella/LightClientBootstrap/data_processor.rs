use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_data_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let max_value = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);
    
    (average, max_value, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,-3.0,false").unwrap();
        writeln!(temp_file, "3,Test3,7.2,true").unwrap();

        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].value, 7.2);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 5.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 15.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 10.0, active: true },
        ];
        
        let (avg, max, count) = calculate_statistics(&records);
        assert_eq!(avg, 10.0);
        assert_eq!(max, 15.0);
        assert_eq!(count, 3);
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidCategory,
    TimestampError,
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value must be positive"),
            ProcessingError::InvalidCategory => write!(f, "Category cannot be empty"),
            ProcessingError::TimestampError => write!(f, "Timestamp must be non-negative"),
            ProcessingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_enabled: bool,
    max_value_limit: Option<f64>,
}

impl DataProcessor {
    pub fn new(validation_enabled: bool) -> Self {
        DataProcessor {
            validation_enabled,
            max_value_limit: None,
        }
    }

    pub fn with_max_value_limit(mut self, limit: f64) -> Self {
        self.max_value_limit = Some(limit);
        self
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if !self.validation_enabled {
            return Ok(());
        }

        if record.value <= 0.0 {
            return Err(ProcessingError::InvalidValue);
        }

        if let Some(limit) = self.max_value_limit {
            if record.value > limit {
                return Err(ProcessingError::InvalidValue);
            }
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::InvalidCategory);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::TimestampError);
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;

        let transformed = DataRecord {
            id: record.id,
            value: record.value * 1.1,
            category: record.category.to_uppercase(),
            timestamp: record.timestamp,
        };

        Ok(transformed)
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());

        for record in records {
            match self.transform_record(&record) {
                Ok(transformed) => processed.push(transformed),
                Err(e) => {
                    if self.validation_enabled {
                        return Err(e);
                    }
                }
            }
        }

        Ok(processed)
    }

    pub fn serialize_records(&self, records: &[DataRecord]) -> Result<String, ProcessingError> {
        serde_json::to_string(records)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }

    pub fn deserialize_records(&self, data: &str) -> Result<Vec<DataRecord>, ProcessingError> {
        serde_json::from_str(data)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(true);
        let record = DataRecord {
            id: 1,
            value: 100.0,
            category: "test".to_string(),
            timestamp: 1234567890,
        };

        let result = processor.transform_record(&record);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.value, 110.0);
        assert_eq!(transformed.category, "TEST");
    }

    #[test]
    fn test_invalid_value_validation() {
        let processor = DataProcessor::new(true);
        let record = DataRecord {
            id: 1,
            value: -10.0,
            category: "test".to_string(),
            timestamp: 1234567890,
        };

        let result = processor.validate_record(&record);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(true).with_max_value_limit(200.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 50.0,
                category: "alpha".to_string(),
                timestamp: 1000,
            },
            DataRecord {
                id: 2,
                value: 150.0,
                category: "beta".to_string(),
                timestamp: 2000,
            },
        ];

        let result = processor.process_batch(records);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].category, "ALPHA");
        assert_eq!(processed[1].category, "BETA");
    }
}