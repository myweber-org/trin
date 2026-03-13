
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            metadata: None,
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        if let Some(metadata) = &mut self.metadata {
            metadata.insert(key.to_string(), value.to_string());
        }
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }

        if self.timestamp < 0 {
            return Err(ProcessingError::ValidationError("Timestamp cannot be negative".to_string()));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::ValidationError("Record must contain at least one value".to_string()));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(ProcessingError::ValidationError("Value key cannot be empty".to_string()));
            }
            if !value.is_finite() {
                return Err(ProcessingError::ValidationError(format!("Value for key '{}' is not finite", key)));
            }
        }

        Ok(())
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) -> Result<(), ProcessingError>
    where
        F: Fn(f64) -> Result<f64, String>,
    {
        let mut transformed_values = HashMap::new();
        
        for (key, value) in &self.values {
            match transform_fn(*value) {
                Ok(transformed) => {
                    transformed_values.insert(key.clone(), transformed);
                }
                Err(err) => {
                    return Err(ProcessingError::TransformationFailed(
                        format!("Failed to transform value for key '{}': {}", key, err)
                    ));
                }
            }
        }
        
        self.values = transformed_values;
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }

        let values: Vec<f64> = self.values.values().copied().collect();
        
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);

        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed_records = Vec::new();
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        
        processed_record.transform_values(|value| {
            if value < 0.0 {
                Err("Negative values not allowed".to_string())
            } else {
                Ok(value * 2.0)
            }
        })?;
        
        processed_records.push(processed_record);
    }
    
    Ok(processed_records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("temperature", 25.5);
        record.add_metadata("source", "sensor_001");
        
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1625097600);
        assert_eq!(record.values.get("temperature"), Some(&25.5));
        assert_eq!(record.metadata.as_ref().unwrap().get("source"), Some(&"sensor_001".to_string()));
    }

    #[test]
    fn test_validation_success() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("temperature", 25.5);
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let record = DataRecord::new(0, 1625097600);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("value", 10.0);
        
        record.transform_values(|v| Ok(v * 2.0)).unwrap();
        assert_eq!(record.values.get("value"), Some(&20.0));
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("a", 10.0);
        record.add_value("b", 20.0);
        record.add_value("c", 30.0);
        
        let stats = record.calculate_statistics();
        
        assert_eq!(stats.get("count"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&60.0));
        assert_eq!(stats.get("mean"), Some(&20.0));
        assert_eq!(stats.get("min"), Some(&10.0));
        assert_eq!(stats.get("max"), Some(&30.0));
    }
}