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
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        Self {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.values.is_empty() {
            return Err(DataError::ValidationFailed("Empty values array".into()));
        }

        let sum: f64 = record.values.iter().sum();
        let avg = sum / record.values.len() as f64;

        if avg.abs() > self.validation_threshold {
            return Err(DataError::ValidationFailed(
                format!("Average value {} exceeds threshold {}", avg, self.validation_threshold)
            ));
        }

        if record.timestamp < 0 {
            return Err(DataError::ValidationFailed("Negative timestamp".into()));
        }

        Ok(())
    }

    pub fn transform_record(&self, mut record: DataRecord) -> Result<DataRecord, DataError> {
        self.validate_record(&record)?;

        for value in record.values.iter_mut() {
            *value *= self.transformation_factor;
        }

        record.metadata.insert(
            "processed".into(),
            chrono::Utc::now().to_rfc3339()
        );

        Ok(record)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>
    ) -> Result<Vec<DataRecord>, (Vec<DataRecord>, DataError)> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            match self.transform_record(record) {
                Ok(transformed) => processed.push(transformed),
                Err(e) => return Err((processed, e)),
            }
        }
        
        Ok(processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(100.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(10.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![100.0, 200.0, 300.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(100.0, 3.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        let transformed = processor.transform_record(record).unwrap();
        assert_eq!(transformed.values, vec![3.0, 6.0, 9.0]);
        assert!(transformed.metadata.contains_key("processed"));
    }
}