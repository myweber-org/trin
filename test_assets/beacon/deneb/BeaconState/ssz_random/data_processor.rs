
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid data value: {0}")]
    InvalidValue(f64),
    #[error("Timestamp out of range: {0}")]
    InvalidTimestamp(i64),
    #[error("Duplicate record ID: {0}")]
    DuplicateId(u32),
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    processed_ids: std::collections::HashSet<u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            processed_ids: std::collections::HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue(record.value));
        }

        if record.timestamp < 0 || record.timestamp > 253402300799 {
            return Err(DataError::InvalidTimestamp(record.timestamp));
        }

        if self.processed_ids.contains(&record.id) {
            return Err(DataError::DuplicateId(record.id));
        }

        self.processed_ids.insert(record.id);
        self.records.push(record);
        Ok(())
    }

    pub fn process_records(&mut self) -> Vec<DataRecord> {
        self.records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        let mut processed = Vec::new();
        for record in &self.records {
            let transformed = DataRecord {
                id: record.id,
                value: record.value * 1.1,
                timestamp: record.timestamp,
            };
            processed.push(transformed);
        }
        
        processed
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
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
        Self {
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

    pub fn process(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            rule.validate(record)?;
        }

        for transformation in &self.transformation_pipeline {
            transformation.apply(record);
        }

        Ok(())
    }

    pub fn batch_process(&self, records: &mut [DataRecord]) -> Vec<Result<(), ProcessingError>> {
        records
            .iter_mut()
            .map(|record| self.process(record))
            .collect()
    }
}

pub trait ValidationRule {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError>;
}

pub trait Transformation {
    fn apply(&self, record: &mut DataRecord);
}

pub struct RequiredFieldValidator {
    field_name: String,
}

impl RequiredFieldValidator {
    pub fn new(field_name: &str) -> Self {
        Self {
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
    field_name: String,
    min_value: f64,
    max_value: f64,
}

impl ValueNormalizer {
    pub fn new(field_name: &str, min_value: f64, max_value: f64) -> Self {
        Self {
            field_name: field_name.to_string(),
            min_value,
            max_value,
        }
    }
}

impl Transformation for ValueNormalizer {
    fn apply(&self, record: &mut DataRecord) {
        if let Some(value) = record.values.get_mut(&self.field_name) {
            let normalized = (*value - self.min_value) / (self.max_value - self.min_value);
            *value = normalized.clamp(0.0, 1.0);
        }
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_validation_rule(RequiredFieldValidator::new("temperature"));
    processor.add_validation_rule(RequiredFieldValidator::new("humidity"));
    
    processor.add_transformation(ValueNormalizer::new("temperature", -20.0, 50.0));
    processor.add_transformation(ValueNormalizer::new("humidity", 0.0, 100.0));
    
    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_field_validation() {
        let validator = RequiredFieldValidator::new("temperature");
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::new(),
            tags: vec![],
        };
        
        assert!(validator.validate(&record).is_err());
        
        record.values.insert("temperature".to_string(), 25.0);
        assert!(validator.validate(&record).is_ok());
    }

    #[test]
    fn test_value_normalization() {
        let normalizer = ValueNormalizer::new("temperature", -20.0, 50.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: {
                let mut map = HashMap::new();
                map.insert("temperature".to_string(), 25.0);
                map
            },
            tags: vec![],
        };

        normalizer.apply(&mut record);
        let normalized_value = record.values.get("temperature").unwrap();
        
        let expected = (25.0 - (-20.0)) / (50.0 - (-20.0));
        assert!((normalized_value - expected).abs() < f64::EPSILON);
    }
}