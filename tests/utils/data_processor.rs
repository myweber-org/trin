
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: u64,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp must be in the past"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.value < 0.0 || record.value > 1000.0 {
        return Err(ValidationError::InvalidValue);
    }
    
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    if record.timestamp > current_time {
        return Err(ValidationError::InvalidTimestamp);
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord, multiplier: f64) -> DataRecord {
    DataRecord {
        id: record.id,
        value: record.value * multiplier,
        timestamp: record.timestamp,
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<DataRecord, ValidationError>> {
    records
        .iter()
        .map(|record| {
            validate_record(record)?;
            Ok(transform_record(record, 1.5))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 100.0,
            timestamp: 1000,
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 100.0,
            timestamp: 1000,
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            value: 100.0,
            timestamp: 1000,
        };
        
        let transformed = transform_record(&record, 2.0);
        assert_eq!(transformed.value, 200.0);
        assert_eq!(transformed.id, record.id);
    }
}