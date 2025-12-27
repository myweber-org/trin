
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

pub struct DataProcessor {
    validation_rules: HashMap<String, ValidationRule>,
    transformation_pipeline: Vec<TransformationStep>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field_name: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct TransformationStep {
    pub name: String,
    pub function: fn(&mut DataRecord) -> Result<(), ProcessingError>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.insert(rule.field_name.clone(), rule);
    }

    pub fn add_transformation_step(&mut self, step: TransformationStep) {
        self.transformation_pipeline.push(step);
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        
        for step in &self.transformation_pipeline {
            (step.function)(&mut record)?;
        }
        
        Ok(record)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        for (field_name, rule) in &self.validation_rules {
            if rule.required && !record.values.contains_key(field_name) {
                return Err(ProcessingError::MissingField(field_name.clone()));
            }
            
            if let Some(&value) = record.values.get(field_name) {
                if let Some(min) = rule.min_value {
                    if value < min {
                        return Err(ProcessingError::OutOfRange(
                            format!("{} below minimum {}", field_name, min)
                        ));
                    }
                }
                
                if let Some(max) = rule.max_value {
                    if value > max {
                        return Err(ProcessingError::OutOfRange(
                            format!("{} above maximum {}", field_name, max)
                        ));
                    }
                }
            }
        }
        
        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidFormat);
        }
        
        Ok(())
    }
}

pub fn normalize_values(record: &mut DataRecord) -> Result<(), ProcessingError> {
    let sum: f64 = record.values.values().sum();
    if sum.abs() < f64::EPSILON {
        return Err(ProcessingError::TransformationFailed(
            "Cannot normalize zero sum".to_string()
        ));
    }
    
    for value in record.values.values_mut() {
        *value /= sum;
    }
    
    Ok(())
}

pub fn add_derived_field(record: &mut DataRecord) -> Result<(), ProcessingError> {
    let count = record.values.len() as f64;
    if count == 0.0 {
        return Err(ProcessingError::TransformationFailed(
            "No values to calculate average".to_string()
        ));
    }
    
    let sum: f64 = record.values.values().sum();
    let average = sum / count;
    
    record.values.insert("average".to_string(), average);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: Some(-50.0),
            max_value: Some(100.0),
            required: true,
        });

        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 25.0);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values,
            tags: vec!["test".to_string()],
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_normalization() {
        let mut values = HashMap::new();
        values.insert("a".to_string(), 1.0);
        values.insert("b".to_string(), 2.0);
        values.insert("c".to_string(), 3.0);
        
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values,
            tags: Vec::new(),
        };

        assert!(normalize_values(&mut record).is_ok());
        
        let sum: f64 = record.values.values().sum();
        assert!((sum - 1.0).abs() < 0.0001);
    }
}