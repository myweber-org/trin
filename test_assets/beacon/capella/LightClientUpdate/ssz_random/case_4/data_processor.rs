
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ProcessingError {
    InvalidInput(String),
    TransformationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ProcessingError> {
        if threshold <= 0.0 {
            return Err(ProcessingError::InvalidInput(
                "Threshold must be positive".to_string(),
            ));
        }
        Ok(DataProcessor { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        if values.is_empty() {
            return Err(ProcessingError::InvalidInput("Empty input array".to_string()));
        }

        let mut result = Vec::with_capacity(values.len());
        for &value in values {
            if value < 0.0 {
                return Err(ProcessingError::InvalidInput(format!(
                    "Negative value encountered: {}",
                    value
                )));
            }

            let processed = self.apply_transformation(value)?;
            result.push(processed);
        }
        Ok(result)
    }

    fn apply_transformation(&self, value: f64) -> Result<f64, ProcessingError> {
        let transformed = (value * value).sqrt() / self.threshold;
        
        if transformed.is_nan() || transformed.is_infinite() {
            Err(ProcessingError::TransformationFailed(
                "Numerical overflow during transformation".to_string(),
            ))
        } else {
            Ok(transformed)
        }
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64), ProcessingError> {
        let processed = self.process_values(values)?;
        
        let sum: f64 = processed.iter().sum();
        let mean = sum / processed.len() as f64;
        
        let variance: f64 = processed
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / processed.len() as f64;
        
        Ok((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processing() {
        let processor = DataProcessor::new(2.0).unwrap();
        let values = vec![1.0, 4.0, 9.0];
        let result = processor.process_values(&values).unwrap();
        assert_eq!(result, vec![0.5, 2.0, 4.5]);
    }

    #[test]
    fn test_invalid_threshold() {
        let result = DataProcessor::new(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(1.0).unwrap();
        let values = vec![2.0, 4.0, 6.0];
        let (mean, std_dev) = processor.calculate_statistics(&values).unwrap();
        assert!((mean - 4.0).abs() < 1e-10);
        assert!((std_dev - 2.0).abs() < 1e-10);
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
    #[error("Value out of range: {0}")]
    OutOfRange(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

pub struct DataProcessor {
    validation_rules: HashMap<String, ValidationRule>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_validation_rule(&mut self, field: String, rule: ValidationRule) {
        self.validation_rules.insert(field, rule);
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<ProcessedRecord, ProcessingError> {
        self.validate_record(record)?;
        
        let normalized_values = self.normalize_values(&record.values);
        let aggregated_value = self.aggregate_values(&normalized_values);
        
        Ok(ProcessedRecord {
            id: record.id,
            timestamp: record.timestamp,
            original_values: record.values.clone(),
            normalized_values,
            aggregated_value,
            metadata: record.metadata.clone(),
        })
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::InvalidFormat);
        }

        for (field, rule) in &self.validation_rules {
            match field.as_str() {
                "timestamp" => {
                    if record.timestamp < rule.min_value as i64 
                        || record.timestamp > rule.max_value as i64 {
                        return Err(ProcessingError::OutOfRange(
                            format!("Timestamp {} out of range", record.timestamp)
                        ));
                    }
                }
                "values" => {
                    for value in &record.values {
                        if *value < rule.min_value || *value > rule.max_value {
                            return Err(ProcessingError::OutOfRange(
                                format!("Value {} out of range", value)
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn normalize_values(&self, values: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return vec![0.0; values.len()];
        }

        values.iter()
            .map(|&v| (v - min) / (max - min))
            .collect()
    }

    fn aggregate_values(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        values.iter().sum::<f64>() / values.len() as f64
    }
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedRecord {
    pub id: u64,
    pub timestamp: i64,
    pub original_values: Vec<f64>,
    pub normalized_values: Vec<f64>,
    pub aggregated_value: f64,
    pub metadata: HashMap<String, String>,
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = processor.normalize_values(&values);
        
        assert_eq!(normalized.len(), 5);
        assert!((normalized[0] - 0.0).abs() < 0.001);
        assert!((normalized[4] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_values() {
        let processor = DataProcessor::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let avg = processor.aggregate_values(&values);
        
        assert!((avg - 3.0).abs() < 0.001);
    }
}