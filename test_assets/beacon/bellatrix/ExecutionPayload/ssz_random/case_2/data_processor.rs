
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
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

pub struct DataProcessor {
    validation_rules: Vec<ValidationRule>,
    transformation_pipeline: Vec<Transformation>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformation_pipeline.push(transformation);
    }

    pub fn process(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate(record)?;
        self.transform(record)
    }

    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            rule.apply(record)?;
        }
        Ok(())
    }

    fn transform(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        for transformation in &self.transformation_pipeline {
            transformed = transformation.apply(&transformed)?;
        }
        
        Ok(transformed)
    }
}

pub trait ValidationRule {
    fn apply(&self, record: &DataRecord) -> Result<(), ProcessingError>;
}

pub trait Transformation {
    fn apply(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError>;
}

pub struct RangeValidation {
    pub min_value: f64,
    pub max_value: f64,
    pub value_index: usize,
}

impl ValidationRule for RangeValidation {
    fn apply(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if let Some(&value) = record.values.get(self.value_index) {
            if value < self.min_value || value > self.max_value {
                return Err(ProcessingError::ValidationFailed(
                    format!("Value {} at index {} is out of range [{}, {}]", 
                           value, self.value_index, self.min_value, self.max_value)
                ));
            }
        }
        Ok(())
    }
}

pub struct NormalizeTransformation {
    pub value_index: usize,
}

impl Transformation for NormalizeTransformation {
    fn apply(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut new_record = record.clone();
        
        if let Some(&value) = record.values.get(self.value_index) {
            let normalized = (value - value.min(0.0)) / (value.max(1.0) - value.min(0.0));
            new_record.values[self.value_index] = normalized;
        }
        
        Ok(new_record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(RangeValidation {
            min_value: 0.0,
            max_value: 100.0,
            value_index: 0,
        });
        
        processor.add_transformation(NormalizeTransformation {
            value_index: 0,
        });
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![50.0, 25.0, 75.0],
            metadata,
        };
        
        let result = processor.process(&record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!(processed.values[0] >= 0.0 && processed.values[0] <= 1.0);
    }
}