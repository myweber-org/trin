
use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData,
    MissingField,
    TransformationFailed,
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::InvalidData => write!(f, "Invalid data provided"),
            ProcessingError::MissingField => write!(f, "Required field is missing"),
            ProcessingError::TransformationFailed => write!(f, "Data transformation failed"),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        DataProcessor {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::InvalidData);
        }

        if record.timestamp <= 0 {
            return Err(ProcessingError::InvalidData);
        }

        if record.values.is_empty() {
            return Err(ProcessingError::MissingField);
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData);
            }
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.iter().any(|&v| v.abs() > self.validation_threshold) {
            return Err(ProcessingError::TransformationFailed);
        }

        for value in record.values.iter_mut() {
            *value *= self.transformation_factor;
        }

        record.metadata.insert(
            "processed".to_string(),
            "true".to_string(),
        );

        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        self.transform_values(&mut record)?;
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        };

        let result = processor.process_record(record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.values, vec![20.0, 40.0, 60.0]);
        assert_eq!(processed.metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_invalid_record_rejection() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: vec![10.0],
            metadata: HashMap::new(),
        };

        let result = processor.process_record(record);
        assert!(result.is_err());
    }
}