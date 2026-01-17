
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::DuplicateTag => write!(f, "Tags contain duplicates"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Self {
        DataRecord {
            id,
            name,
            value,
            tags,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let mut seen = HashMap::new();
        for tag in &self.tags {
            if seen.insert(tag, true).is_some() {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    pub fn get_summary(&self) -> String {
        format!(
            "Record {}: {} (value: {:.2}) with {} tags",
            self.id,
            self.name,
            self.value,
            self.tags.len()
        )
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Vec<String> {
    let mut results = Vec::new();
    
    for record in records {
        if let Err(e) = record.validate() {
            results.push(format!("Validation failed: {}", e));
            continue;
        }
        
        record.transform(multiplier);
        results.push(record.get_summary());
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(
            1,
            "test".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag2".to_string()]
        );
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "test".to_string(),
            100.0,
            vec!["tag1".to_string()]
        );
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform() {
        let mut record = DataRecord::new(
            1,
            "test".to_string(),
            100.0,
            vec!["tag1".to_string()]
        );
        
        record.transform(2.0);
        assert_eq!(record.value, 200.0);
        assert_eq!(record.name, "TEST");
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u64, value: f64, timestamp: i64, category: &str) -> Self {
        Self {
            id,
            value,
            timestamp,
            category: category.to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("ID cannot be zero".into());
        }
        
        if self.value.is_nan() || self.value.is_infinite() {
            return Err("Value must be a finite number".into());
        }
        
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".into());
        }
        
        if self.category.is_empty() {
            return Err("Category cannot be empty".into());
        }
        
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let processed_record = DataRecord {
            value: record.value * 1.1,
            ..record
        };
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn filter_by_category(records: Vec<DataRecord>, category: &str) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|r| r.category == category)
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, 1234567890, "test");
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 42.5, 1234567890, "test");
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 10.0, 1234567890, "A"),
            DataRecord::new(2, 20.0, 1234567891, "B"),
        ];
        
        let processed = process_records(records).unwrap();
        assert_eq!(processed[0].value, 11.0);
        assert_eq!(processed[1].value, 22.0);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            DataRecord::new(1, 10.0, 1234567890, "A"),
            DataRecord::new(2, 20.0, 1234567891, "B"),
            DataRecord::new(3, 30.0, 1234567892, "A"),
        ];
        
        let filtered = filter_by_category(records, "A");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, 1234567890, "A"),
            DataRecord::new(2, 20.0, 1234567891, "B"),
            DataRecord::new(3, 30.0, 1234567892, "C"),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}