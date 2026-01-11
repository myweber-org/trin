use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Transformation failed")]
    TransformationFailed,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationError("Timestamp cannot be negative".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationError("Values cannot be empty".to_string()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationError("Key cannot be empty".to_string()));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationError(format!("Value for {} is not finite", key)));
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(DataError::TransformationFailed);
        }
        
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
        
        Ok(())
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    pub fn calculate_sum(&self) -> f64 {
        self.values.values().sum()
    }
    
    pub fn calculate_average(&self) -> Option<f64> {
        let count = self.values.len() as f64;
        if count > 0.0 {
            Some(self.calculate_sum() / count)
        } else {
            None
        }
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        record.transform(multiplier)?;
        record.add_tag("processed".to_string());
        processed.push(record.clone());
    }
    
    Ok(processed)
}

pub fn filter_records_by_threshold(records: &[DataRecord], threshold: f64) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.calculate_sum() > threshold)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("temp".to_string(), 25.5)]),
            tags: vec![],
        };
        
        assert!(record.validate().is_ok());
        
        record.id = 0;
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("value".to_string(), 10.0)]),
            tags: vec![],
        };
        
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.values.get("value"), Some(&20.0));
    }
    
    #[test]
    fn test_calculate_average() {
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("a".to_string(), 10.0),
                ("b".to_string(), 20.0),
                ("c".to_string(), 30.0),
            ]),
            tags: vec![],
        };
        
        assert_eq!(record.calculate_average(), Some(20.0));
    }
}