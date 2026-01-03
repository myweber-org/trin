
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
                return Err(DataError::ValidationFailed(
                    format!("Value for key '{}' must be finite", key)
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> &mut Self {
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
        self
    }
    
    pub fn add_tag(&mut self, tag: String) -> &mut Self {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }
}

pub fn process_records(
    records: Vec<DataRecord>,
    multiplier: f64,
) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for mut record in records {
        record.validate()?;
        record.transform(multiplier);
        processed.push(record);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    for record in records {
        for (key, value) in &record.values {
            let entry = stats.entry(key.clone()).or_insert(Vec::new());
            entry.push(*value);
        }
    }
    
    let mut result = HashMap::new();
    for (key, values) in stats {
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        result.insert(format!("{}_mean", key), mean);
        result.insert(format!("{}_variance", key), variance);
        result.insert(format!("{}_count", key), count);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("temp".into(), 25.5)]),
            tags: vec!["sensor".into()],
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
            values: HashMap::from([("value".into(), 10.0)]),
            tags: vec![],
        };
        
        record.transform(2.0);
        assert_eq!(record.values.get("value"), Some(&20.0));
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: HashMap::from([("a".into(), 1.0)]),
                tags: vec![],
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: HashMap::from([("b".into(), 2.0)]),
                tags: vec![],
            },
        ];
        
        let processed = process_records(records, 3.0).unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values.get("a"), Some(&3.0));
        assert_eq!(processed[1].values.get("b"), Some(&6.0));
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        DataRecord {
            id,
            value,
            category: category.to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), String> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.add_record(record)?;
        }

        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.records.iter().map(|r| r.value).max_by(|a, b| a.partial_cmp(b).unwrap());

        (count, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "test");
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, -5.0, "");
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        assert!(processor.add_record(DataRecord::new(1, 10.0, "A")).is_ok());
        assert!(processor.add_record(DataRecord::new(2, 20.0, "B")).is_ok());
        
        assert_eq!(processor.calculate_average(), Some(15.0));
        assert_eq!(processor.filter_by_category("A").len(), 1);
        
        let (count, avg, max) = processor.get_statistics();
        assert_eq!(count, 2);
        assert_eq!(avg, Some(15.0));
        assert_eq!(max, Some(20.0));
    }
}