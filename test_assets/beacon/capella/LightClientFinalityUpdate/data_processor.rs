
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
            return Err(DataError::ValidationFailed("ID cannot be zero".into()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed("Timestamp cannot be negative".into()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationFailed("Values cannot be empty".into()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed("Key cannot be empty".into()));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(format!("Value for {} is not finite", key)));
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) {
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.transform(multiplier);
        processed_record.add_tag("processed".into());
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    for key in records[0].values.keys() {
        let values: Vec<f64> = records
            .iter()
            .filter_map(|r| r.values.get(key))
            .copied()
            .collect();
        
        if !values.is_empty() {
            let count = values.len() as f64;
            let sum: f64 = values.iter().sum();
            let mean = sum / count;
            let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count;
            
            stats.insert(format!("{}_mean", key), mean);
            stats.insert(format!("{}_variance", key), variance);
            stats.insert(format!("{}_count", key), count);
        }
    }
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut valid_record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: HashMap::from([("temp".into(), 25.5)]),
            tags: vec![],
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1000,
            values: HashMap::from([("temp".into(), 25.5)]),
            tags: vec![],
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: HashMap::from([("value".into(), 10.0)]),
            tags: vec![],
        };
        
        record.transform(2.0);
        assert_eq!(record.values.get("value"), Some(&20.0));
    }
    
    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: HashMap::from([("data".into(), 5.0)]),
                tags: vec![],
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: HashMap::from([("data".into(), 10.0)]),
                tags: vec![],
            },
        ];
        
        let result = process_records(&mut records, 3.0);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert!(processed[0].tags.contains(&"processed".into()));
    }
}