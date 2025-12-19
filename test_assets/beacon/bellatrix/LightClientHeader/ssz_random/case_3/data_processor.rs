
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidName,
    InvalidValue,
    InvalidCategory,
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidName => write!(f, "Invalid record name"),
            DataError::InvalidValue => write!(f, "Invalid value"),
            DataError::InvalidCategory => write!(f, "Invalid category"),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            category,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(DataError::InvalidName);
        }
        
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(DataError::InvalidValue);
        }
        
        if self.category.trim().is_empty() {
            return Err(DataError::InvalidCategory);
        }
        
        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::new();
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        
        if processed_record.value < 0.0 {
            processed_record.value = 0.0;
        }
        
        processed_record.name = processed_record.name.trim().to_uppercase();
        processed_record.category = processed_record.category.trim().to_lowercase();
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

pub fn filter_by_category(records: &[DataRecord], category: &str) -> Vec<DataRecord> {
    records.iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "category".to_string());
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, "".to_string(), 10.5, "category".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord::new(1, "  test one  ".to_string(), -5.0, "CATEGORY".to_string()),
            DataRecord::new(2, "test two".to_string(), 15.0, "another".to_string()),
        ];
        
        let processed = process_records(&mut records).unwrap();
        assert_eq!(processed[0].name, "TEST ONE");
        assert_eq!(processed[0].value, 0.0);
        assert_eq!(processed[0].category, "category");
        assert_eq!(processed[1].name, "TEST TWO");
        assert_eq!(processed[1].value, 15.0);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, "a".to_string(), 10.0, "cat".to_string()),
            DataRecord::new(2, "b".to_string(), 20.0, "cat".to_string()),
            DataRecord::new(3, "c".to_string(), 30.0, "cat".to_string()),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            DataRecord::new(1, "a".to_string(), 10.0, "cat1".to_string()),
            DataRecord::new(2, "b".to_string(), 20.0, "cat2".to_string()),
            DataRecord::new(3, "c".to_string(), 30.0, "cat1".to_string()),
        ];
        
        let filtered = filter_by_category(&records, "cat1");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }
}