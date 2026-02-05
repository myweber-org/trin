
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
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
}

pub fn validate_timestamp(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.timestamp < 0 {
        return Err(ProcessingError::ValidationError(
            "Timestamp cannot be negative".to_string(),
        ));
    }
    Ok(())
}

pub fn normalize_values(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    if record.values.is_empty() {
        return Err(ProcessingError::TransformationError(
            "No values to normalize".to_string(),
        ));
    }

    let sum: f64 = record.values.iter().sum();
    if sum.abs() < f64::EPSILON {
        return Err(ProcessingError::TransformationError(
            "Cannot normalize zero vector".to_string(),
        ));
    }

    let normalized_values: Vec<f64> = record.values.iter().map(|&v| v / sum).collect();

    Ok(DataRecord {
        values: normalized_values,
        ..record
    })
}

pub fn add_processing_metadata(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let mut new_metadata = record.metadata.clone();
    new_metadata.insert(
        "processed_at".to_string(),
        chrono::Utc::now().timestamp().to_string(),
    );
    new_metadata.insert("record_id".to_string(), record.id.to_string());

    Ok(DataRecord {
        metadata: new_metadata,
        ..record
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_timestamp() {
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let invalid_record = DataRecord {
            id: 2,
            timestamp: -1,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(validate_timestamp(&valid_record).is_ok());
        assert!(validate_timestamp(&invalid_record).is_err());
    }

    #[test]
    fn test_normalize_values() {
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let normalized = normalize_values(record).unwrap();
        let sum: f64 = normalized.values.iter().sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_processor_pipeline() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(validate_timestamp);
        processor.add_transformation(normalize_values);
        processor.add_transformation(add_processing_metadata);

        let record = DataRecord {
            id: 42,
            timestamp: 1609459200,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert!(processed.metadata.contains_key("processed_at"));
        assert!(processed.metadata.contains_key("record_id"));
    }
}