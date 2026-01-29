
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value must be positive"),
            ProcessingError::InvalidTimestamp => write!(f, "Timestamp cannot be negative"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value <= 0.0 {
        return Err(ProcessingError::InvalidValue);
    }
    
    if record.timestamp < 0 {
        return Err(ProcessingError::InvalidTimestamp);
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> DataRecord {
    DataRecord {
        id: record.id,
        value: record.value * 2.0,
        timestamp: record.timestamp + 3600,
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        validate_record(&record)?;
        let transformed = transform_record(&record);
        processed.push(transformed);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1625097600,
        };
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_record_invalid_value() {
        let record = DataRecord {
            id: 1,
            value: -5.0,
            timestamp: 1625097600,
        };
        assert!(matches!(validate_record(&record), Err(ProcessingError::InvalidValue)));
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            value: 10.0,
            timestamp: 1000,
        };
        let transformed = transform_record(&record);
        assert_eq!(transformed.value, 20.0);
        assert_eq!(transformed.timestamp, 4600);
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord { id: 1, value: 5.0, timestamp: 1000 },
            DataRecord { id: 2, value: 15.0, timestamp: 2000 },
        ];
        
        let result = process_records(records);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].value, 10.0);
        assert_eq!(processed[1].value, 30.0);
    }
}