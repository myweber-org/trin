
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data value: {0}")]
    InvalidValue(String),
    #[error("Timestamp out of range")]
    InvalidTimestamp,
    #[error("Serialization error")]
    SerializationFailed,
}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        DataProcessor { min_value, max_value }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(
                format!("Value {} outside range [{}, {}]", record.value, self.min_value, self.max_value)
            ));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        Ok(())
    }

    pub fn normalize_value(&self, record: &DataRecord) -> f64 {
        (record.value - self.min_value) / (self.max_value - self.min_value)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<f64>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let normalized = self.normalize_value(&record);
            results.push(normalized);
        }
        
        Ok(results)
    }

    pub fn serialize_records(records: &[DataRecord]) -> Result<String, ProcessingError> {
        serde_json::to_string(records)
            .map_err(|_| ProcessingError::SerializationFailed)
    }

    pub fn deserialize_records(data: &str) -> Result<Vec<DataRecord>, ProcessingError> {
        serde_json::from_str(data)
            .map_err(|_| ProcessingError::SerializationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new(0.0, 100.0);
        let valid_record = DataRecord { id: 1, value: 50.0, timestamp: 1234567890 };
        let invalid_record = DataRecord { id: 2, value: 150.0, timestamp: 1234567890 };
        
        assert!(processor.validate_record(&valid_record).is_ok());
        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord { id: 1, value: 75.0, timestamp: 1234567890 };
        
        assert_eq!(processor.normalize_value(&record), 0.75);
    }

    #[test]
    fn test_serialization() {
        let records = vec![
            DataRecord { id: 1, value: 25.0, timestamp: 1234567890 },
            DataRecord { id: 2, value: 75.0, timestamp: 1234567891 },
        ];
        
        let serialized = DataProcessor::serialize_records(&records);
        assert!(serialized.is_ok());
        
        let deserialized = DataProcessor::deserialize_records(&serialized.unwrap());
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap().len(), 2);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }

        Ok(DataProcessor { threshold })
    }

    pub fn process_data(&self, input: Vec<f64>) -> Result<Vec<f64>, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }

        let filtered_data: Vec<f64> = input
            .into_iter()
            .filter(|&value| value >= self.threshold)
            .collect();

        if filtered_data.is_empty() {
            return Err(ValidationError {
                message: format!("No data points above threshold {}", self.threshold),
            });
        }

        Ok(filtered_data)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> (f64, f64, f64) {
        let count = data.len() as f64;
        let sum: f64 = data.iter().sum();
        let mean = sum / count;

        let variance: f64 = data
            .iter()
            .map(|value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        if data.is_empty() {
            return Vec::new();
        }

        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;

        if range == 0.0 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&value| (value - min) / range)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_data() {
        let processor = DataProcessor::new(0.3).unwrap();
        let input = vec![0.1, 0.4, 0.2, 0.5, 0.6];
        let result = processor.process_data(input).unwrap();
        assert_eq!(result, vec![0.4, 0.5, 0.6]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&data);
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }

    #[test]
    fn test_normalize_data() {
        let processor = DataProcessor::new(0.0).unwrap();
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let normalized = processor.normalize_data(&data);
        assert_eq!(normalized, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
    }
}