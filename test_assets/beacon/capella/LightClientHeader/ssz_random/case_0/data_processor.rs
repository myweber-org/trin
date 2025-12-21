
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
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

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::InvalidValue);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        if record.value > self.threshold {
            return Err(ProcessingError::ValidationFailed(
                format!("Value {} exceeds threshold {}", record.value, self.threshold)
            ));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> DataRecord {
        DataRecord {
            id: record.id,
            value: record.value * 2.0,
            timestamp: record.timestamp + 3600,
        }
    }

    pub fn process_records(&self, records: &[DataRecord]) -> Vec<Result<DataRecord, ProcessingError>> {
        records.iter()
            .map(|record| {
                self.validate_record(record)?;
                Ok(self.transform_record(record))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1609459200,
        };

        let result = processor.process_records(&[record.clone()]);
        assert!(result[0].is_ok());
        
        let processed = result[0].as_ref().unwrap();
        assert_eq!(processed.value, 100.0);
        assert_eq!(processed.timestamp, 1609459200 + 3600);
    }

    #[test]
    fn test_invalid_value() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1609459200,
        };

        let result = processor.process_records(&[record]);
        assert!(result[0].is_err());
    }

    #[test]
    fn test_threshold_exceeded() {
        let processor = DataProcessor::new(50.0);
        let record = DataRecord {
            id: 1,
            value: 75.0,
            timestamp: 1609459200,
        };

        let result = processor.process_records(&[record]);
        assert!(result[0].is_err());
    }
}