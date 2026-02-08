
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if data.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }

        let processed: Vec<f64> = data
            .iter()
            .map(|&x| x * 2.0)
            .filter(|&x| x > 0.0)
            .collect();

        if processed.is_empty() {
            return Err("All values filtered out".to_string());
        }

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn get_cached_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn calculate_statistics(data: &[f64]) -> (f64, f64, f64) {
        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_valid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        let result = processor.process_numeric_data("test", data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_process_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![f64::NAN, 1.0];
        let result = processor.process_numeric_data("test", data);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = DataProcessor::calculate_statistics(&data);
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
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
        let mut processed_record = record.clone();

        for rule in &self.validation_rules {
            rule.validate(&processed_record)?;
        }

        for transformation in &self.transformation_pipeline {
            processed_record = transformation.apply(&processed_record)?;
        }

        Ok(processed_record)
    }

    pub fn batch_process(
        &self,
        records: &[DataRecord],
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        records
            .iter()
            .map(|record| self.process(record))
            .collect()
    }
}

pub trait ValidationRule {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError>;
}

pub trait Transformation {
    fn apply(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError>;
}

pub struct RequiredFieldValidator {
    field_name: String,
}

impl RequiredFieldValidator {
    pub fn new(field_name: &str) -> Self {
        RequiredFieldValidator {
            field_name: field_name.to_string(),
        }
    }
}

impl ValidationRule for RequiredFieldValidator {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if !record.values.contains_key(&self.field_name) {
            return Err(ProcessingError::MissingField(self.field_name.clone()));
        }
        Ok(())
    }
}

pub struct ValueNormalizer {
    target_field: String,
    scale_factor: f64,
}

impl ValueNormalizer {
    pub fn new(target_field: &str, scale_factor: f64) -> Self {
        ValueNormalizer {
            target_field: target_field.to_string(),
            scale_factor,
        }
    }
}

impl Transformation for ValueNormalizer {
    fn apply(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut new_record = record.clone();
        
        if let Some(value) = new_record.values.get_mut(&self.target_field) {
            *value *= self.scale_factor;
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
        
        processor.add_validation_rule(RequiredFieldValidator::new("temperature"));
        processor.add_transformation(ValueNormalizer::new("temperature", 0.556));

        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 100.0);
        values.insert("humidity".to_string(), 50.0);

        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values,
            tags: vec!["sensor".to_string(), "room1".to_string()],
        };

        let result = processor.process(&record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!((processed.values["temperature"] - 55.6).abs() < 0.001);
    }
}