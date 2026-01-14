
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
            ProcessingError::InvalidValue => write!(f, "Invalid numeric value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value.is_nan() || record.value.is_infinite() {
        return Err(ProcessingError::InvalidValue);
    }
    
    if record.timestamp < 0 {
        return Err(ProcessingError::InvalidTimestamp);
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord, multiplier: f64) -> Result<DataRecord, ProcessingError> {
    validate_record(record)?;
    
    let transformed_value = record.value * multiplier;
    
    if transformed_value > 1000.0 {
        return Err(ProcessingError::ValidationFailed(
            "Transformed value exceeds maximum limit".to_string()
        ));
    }
    
    Ok(DataRecord {
        id: record.id,
        value: transformed_value,
        timestamp: record.timestamp,
    })
}

pub fn process_records(records: &[DataRecord], multiplier: f64) -> Vec<Result<DataRecord, ProcessingError>> {
    records
        .iter()
        .map(|record| transform_record(record, multiplier))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1672531200,
        };
        
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_value() {
        let record = DataRecord {
            id: 2,
            value: f64::NAN,
            timestamp: 1672531200,
        };
        
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_within_limit() {
        let record = DataRecord {
            id: 3,
            value: 50.0,
            timestamp: 1672531200,
        };
        
        let result = transform_record(&record, 2.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, 100.0);
    }

    #[test]
    fn test_transform_exceeds_limit() {
        let record = DataRecord {
            id: 4,
            value: 600.0,
            timestamp: 1672531200,
        };
        
        let result = transform_record(&record, 2.0);
        assert!(result.is_err());
    }
}