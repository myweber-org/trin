
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field_name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<HashMap<String, f64>>, String> {
        let mut processed = Vec::new();

        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(validated_record) => {
                    let transformed = self.transform_record(&validated_record);
                    processed.push(transformed);
                    self.cache_record(index, &transformed);
                }
                Err(e) => return Err(format!("Validation failed at record {}: {}", index, e)),
            }
        }

        Ok(processed)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<HashMap<String, f64>, String> {
        let mut validated = HashMap::new();

        for rule in &self.validation_rules {
            match record.get(&rule.field_name) {
                Some(&value) => {
                    if value < rule.min_value || value > rule.max_value {
                        return Err(format!(
                            "Field '{}' value {} out of range [{}, {}]",
                            rule.field_name, value, rule.min_value, rule.max_value
                        ));
                    }
                    validated.insert(rule.field_name.clone(), value);
                }
                None => {
                    if rule.required {
                        return Err(format!("Required field '{}' missing", rule.field_name));
                    }
                }
            }
        }

        Ok(validated)
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut transformed = HashMap::new();

        for (key, value) in record {
            let transformed_value = match key.as_str() {
                "temperature" => (value - 32.0) * 5.0 / 9.0,
                "pressure" => value * 1000.0,
                "humidity" => value.min(100.0).max(0.0),
                _ => *value,
            };
            transformed.insert(key.clone(), transformed_value);
        }

        transformed
    }

    fn cache_record(&mut self, index: usize, record: &HashMap<String, f64>) {
        for (key, value) in record {
            self.cache
                .entry(key.clone())
                .or_insert_with(Vec::new)
                .push(*value);
        }
    }

    pub fn get_cached_stats(&self, field: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(field).map(|values| {
            let count = values.len() as f64;
            let sum: f64 = values.iter().sum();
            let mean = sum / count;
            let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count;
            let std_dev = variance.sqrt();

            (mean, variance, std_dev)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 150.0,
            required: true,
        });

        let dataset = vec![
            [("temperature".to_string(), 68.0)].iter().cloned().collect(),
            [("temperature".to_string(), 32.0)].iter().cloned().collect(),
        ];

        let result = processor.process_dataset(&dataset);
        assert!(result.is_ok());
        
        if let Ok(processed) = result {
            assert_eq!(processed.len(), 2);
            assert!((processed[0]["temperature"] - 20.0).abs() < 0.001);
        }
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
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

    pub fn process(&self, record: &mut DataRecord) -> Result<(), DataError> {
        for rule in &self.validation_rules {
            rule.validate(record)?;
        }

        for transformation in &self.transformation_pipeline {
            transformation.apply(record);
        }

        Ok(())
    }

    pub fn batch_process(&self, records: &mut [DataRecord]) -> Vec<Result<(), DataError>> {
        records
            .iter_mut()
            .map(|record| self.process(record))
            .collect()
    }
}

pub trait ValidationRule {
    fn validate(&self, record: &DataRecord) -> Result<(), DataError>;
}

pub trait Transformation {
    fn apply(&self, record: &mut DataRecord);
}

pub struct RequiredFieldRule {
    field_name: String,
}

impl RequiredFieldRule {
    pub fn new(field_name: &str) -> Self {
        RequiredFieldRule {
            field_name: field_name.to_string(),
        }
    }
}

impl ValidationRule for RequiredFieldRule {
    fn validate(&self, record: &DataRecord) -> Result<(), DataError> {
        if !record.values.contains_key(&self.field_name) {
            return Err(DataError::MissingField(self.field_name.clone()));
        }
        Ok(())
    }
}

pub struct NormalizeTransformation {
    factor: f64,
}

impl NormalizeTransformation {
    pub fn new(factor: f64) -> Self {
        NormalizeTransformation { factor }
    }
}

impl Transformation for NormalizeTransformation {
    fn apply(&self, record: &mut DataRecord) {
        for value in record.values.values_mut() {
            *value /= self.factor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_field_validation() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(RequiredFieldRule::new("temperature"));

        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::new(),
            tags: vec![],
        };

        let result = processor.process(&mut record);
        assert!(result.is_err());

        record.values.insert("temperature".to_string(), 25.5);
        let result = processor.process(&mut record);
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalize_transformation() {
        let mut processor = DataProcessor::new();
        processor.add_transformation(NormalizeTransformation::new(10.0));

        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("value1".to_string(), 100.0),
                ("value2".to_string(), 50.0),
            ]),
            tags: vec![],
        };

        processor.process(&mut record).unwrap();
        assert_eq!(record.values.get("value1"), Some(&10.0));
        assert_eq!(record.values.get("value2"), Some(&5.0));
    }
}