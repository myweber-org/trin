
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.id == 0 {
        return Err(ProcessingError::ValidationFailed("ID cannot be zero".into()));
    }
    
    if record.timestamp < 0 {
        return Err(ProcessingError::ValidationFailed("Timestamp cannot be negative".into()));
    }
    
    if record.values.is_empty() {
        return Err(ProcessingError::ValidationFailed("Values cannot be empty".into()));
    }
    
    Ok(())
}

pub fn normalize_values(record: &mut DataRecord) -> Result<(), ProcessingError> {
    if record.values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
        return Err(ProcessingError::TransformationError("Invalid numeric values".into()));
    }
    
    let min = record.values
        .iter()
        .fold(f64::INFINITY, |a, &b| a.min(b));
    let max = record.values
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    if (max - min).abs() < f64::EPSILON {
        return Err(ProcessingError::TransformationError("Cannot normalize constant values".into()));
    }
    
    for value in &mut record.values {
        *value = (*value - min) / (max - min);
    }
    
    Ok(())
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<DataRecord, ProcessingError>> {
    records
        .iter_mut()
        .map(|record| {
            validate_record(record)
                .and_then(|_| normalize_values(record))
                .map(|_| record.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(validate_record(&record).is_ok());
        assert!(normalize_values(&mut record).is_ok());
        assert_eq!(record.values[0], 0.0);
        assert_eq!(record.values[2], 1.0);
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: vec![1.0],
            metadata: HashMap::new(),
        };
        
        assert!(validate_record(&record).is_err());
    }
}