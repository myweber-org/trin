
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
use std::collections::HashMap;

#[derive(Debug, Clone)]
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

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.id == 0 {
            return Err("ID cannot be zero");
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative");
        }
        if self.values.is_empty() {
            return Err("Record must contain at least one value");
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count as f64;
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some(DataStatistics {
            count,
            sum,
            mean,
            min,
            max,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

pub fn process_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| record.validate().is_ok())
        .filter(|record| {
            if let Some(stats) = record.calculate_statistics() {
                stats.mean.is_finite() && !stats.mean.is_nan()
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

pub fn transform_records(records: &[DataRecord], multiplier: f64) -> Vec<DataRecord> {
    records
        .iter()
        .map(|record| {
            let mut transformed = record.clone();
            transformed.values = transformed
                .values
                .iter()
                .map(|&v| v * multiplier)
                .collect();
            transformed
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1000);
        record.add_value(42.5);
        
        assert!(record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1000);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1000);
        record.add_value(10.0).add_value(20.0).add_value(30.0);
        
        let stats = record.calculate_statistics().unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }

    #[test]
    fn test_record_processing() {
        let mut valid_record = DataRecord::new(1, 1000);
        valid_record.add_value(5.0);
        
        let mut invalid_record = DataRecord::new(0, 1000);
        invalid_record.add_value(10.0);
        
        let records = vec![valid_record.clone(), invalid_record];
        let processed = process_records(&records);
        
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].id, 1);
    }
}