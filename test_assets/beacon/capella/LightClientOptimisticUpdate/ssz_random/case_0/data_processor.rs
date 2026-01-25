
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::InvalidFormat);
        }

        if self.timestamp < 0 {
            return Err(ProcessingError::OutOfRange("timestamp".to_string()));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::MissingField("values".to_string()));
        }

        Ok(())
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }

        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn normalize_values(&mut self) {
        if let Some(avg) = self.calculate_average() {
            if avg != 0.0 {
                for value in self.values.iter_mut() {
                    *value /= avg;
                }
            }
        }
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records.iter_mut() {
        record.validate()?;
        record.normalize_values();
        processed.push(record.clone());
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value(10.0);
        record.add_value(20.0);

        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, -1);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_average_calculation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value(10.0);
        record.add_value(20.0);
        record.add_value(30.0);

        assert_eq!(record.calculate_average(), Some(20.0));
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value(10.0);
        record.add_value(20.0);
        record.add_value(30.0);

        record.normalize_values();
        let normalized: Vec<f64> = record.values.iter().map(|v| (v * 100.0).round() / 100.0).collect();
        
        assert_eq!(normalized, vec![0.5, 1.0, 1.5]);
    }
}