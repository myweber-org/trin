
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

#[derive(Debug, Error)]
pub enum ProcessingError {
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

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Empty values array".to_string(),
            ));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationFailed(
                    "Invalid numeric value".to_string(),
                ));
            }

            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationFailed(format!(
                    "Value {} exceeds threshold {}",
                    value, self.validation_threshold
                )));
            }
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.iter().any(|v| v.is_nan()) {
            return Err(ProcessingError::TransformationError(
                "Cannot transform NaN values".to_string(),
            ));
        }

        for value in &mut record.values {
            *value *= self.transformation_factor;
            *value = value.round();
        }

        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        self.transform_values(&mut record)?;

        record.metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string(),
        );
        record.metadata.insert(
            "processor_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );

        Ok(record)
    }
}

pub fn create_sample_record() -> DataRecord {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "sensor_001".to_string());
    metadata.insert("unit".to_string(), "celsius".to_string());

    DataRecord {
        id: 12345,
        timestamp: chrono::Utc::now().timestamp(),
        values: vec![25.5, 26.1, 24.8, 25.9],
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(100.0, 2.0);
        let record = create_sample_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(10.0, 2.0);
        let record = create_sample_record();
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = DataProcessor::new(100.0, 2.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.5, 2.5, 3.5],
            metadata: HashMap::new(),
        };

        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values, vec![3.0, 5.0, 7.0]);
    }

    #[test]
    fn test_full_processing() {
        let processor = DataProcessor::new(100.0, 2.0);
        let record = create_sample_record();
        let result = processor.process_record(record);

        assert!(result.is_ok());
        let processed = result.unwrap();
        assert!(processed.metadata.contains_key("processed_timestamp"));
        assert!(processed.metadata.contains_key("processor_version"));
    }
}