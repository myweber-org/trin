
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidTimestamp(i64),
    EmptyCategory,
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidTimestamp(t) => write!(f, "Invalid timestamp: {}", t),
            ProcessingError::EmptyCategory => write!(f, "Category cannot be empty"),
            ProcessingError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_enabled: bool,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(validation_enabled: bool, max_value: f64) -> Self {
        DataProcessor {
            validation_enabled,
            max_value,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if self.validation_enabled {
            if record.value < 0.0 || record.value > self.max_value {
                return Err(ProcessingError::InvalidValue(record.value));
            }
            
            if record.timestamp < 0 {
                return Err(ProcessingError::InvalidTimestamp(record.timestamp));
            }
            
            if record.category.trim().is_empty() {
                return Err(ProcessingError::EmptyCategory);
            }
        }
        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> DataRecord {
        DataRecord {
            id: record.id,
            value: record.value * 1.1,
            timestamp: record.timestamp + 3600,
            category: record.category.to_uppercase(),
        }
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record);
            processed.push(transformed);
        }
        
        Ok(processed)
    }

    pub fn serialize_to_json(&self, records: &[DataRecord]) -> Result<String, ProcessingError> {
        serde_json::to_string(records)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 1,
            value: 500.0,
            timestamp: 1625097600,
            category: "analytics".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
        
        let transformed = processor.transform_record(&record);
        assert_eq!(transformed.value, 550.0);
        assert_eq!(transformed.timestamp, 1625101200);
        assert_eq!(transformed.category, "ANALYTICS");
    }

    #[test]
    fn test_invalid_value() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 1,
            value: 1500.0,
            timestamp: 1625097600,
            category: "test".to_string(),
        };
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(ProcessingError::InvalidValue(1500.0))
        ));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(true, 1000.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 100.0,
                timestamp: 1625097600,
                category: "alpha".to_string(),
            },
            DataRecord {
                id: 2,
                value: 200.0,
                timestamp: 1625097600,
                category: "beta".to_string(),
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