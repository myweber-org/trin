
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    MetadataTooLarge,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        if self.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }
        
        if self.metadata.len() > 100 {
            return Err(ValidationError::MetadataTooLarge);
        }
        
        Ok(())
    }
    
    pub fn normalize_values(&mut self) {
        if self.values.is_empty() {
            return;
        }
        
        let sum: f64 = self.values.iter().sum();
        let mean = sum / self.values.len() as f64;
        
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        if std_dev > 0.0 {
            for value in self.values.iter_mut() {
                *value = (*value - mean) / std_dev;
            }
        }
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<(), ValidationError>> {
    records
        .iter_mut()
        .map(|record| {
            record.normalize_values();
            record.validate()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(record.validate().is_ok());
        
        record.normalize_values();
        assert_eq!(record.values.len(), 3);
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            timestamp: 1625097600,
            values: vec![1.0],
            metadata: HashMap::new(),
        };
        
        assert_eq!(record.validate(), Err(ValidationError::InvalidId));
    }
}