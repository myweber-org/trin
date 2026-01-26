
use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records.into_iter().map(|record| self.process(record)).collect()
    }
}

pub fn validate_timestamp(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.timestamp <= 0 {
        return Err(ProcessingError::ValidationError(
            "Timestamp must be positive".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_values_length(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.values.is_empty() {
        return Err(ProcessingError::ValidationError(
            "Values array cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn normalize_values(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let sum: f64 = record.values.iter().sum();
    if sum.abs() < f64::EPSILON {
        return Err(ProcessingError::TransformationFailed(
            "Cannot normalize zero vector".to_string(),
        ));
    }

    let normalized_values: Vec<f64> = record.values.iter().map(|&v| v / sum).collect();

    Ok(DataRecord {
        values: normalized_values,
        ..record
    })
}

pub fn add_processing_timestamp(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let processing_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| {
            ProcessingError::TransformationFailed("Failed to get system time".to_string())
        })?
        .as_secs() as i64;

    let mut metadata = record.metadata;
    metadata.insert("processed_at".to_string(), processing_time.to_string());

    Ok(DataRecord {
        metadata,
        ..record
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_rules() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(validate_timestamp);
        processor.add_validation_rule(validate_values_length);

        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let invalid_record = DataRecord {
            id: 2,
            timestamp: -1,
            values: vec![],
            metadata: HashMap::new(),
        };

        assert!(processor.process(valid_record).is_ok());
        assert!(processor.process(invalid_record).is_err());
    }

    #[test]
    fn test_transformation_pipeline() {
        let mut processor = DataProcessor::new();
        processor.add_transformation(normalize_values);
        processor.add_transformation(add_processing_timestamp);

        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(result.is_ok());

        let processed = result.unwrap();
        let sum: f64 = processed.values.iter().sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
        assert!(processed.metadata.contains_key("processed_at"));
    }
}