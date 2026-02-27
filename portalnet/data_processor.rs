
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
pub enum ValidationError {
    InvalidId,
    InvalidName,
    InvalidValue,
    InvalidCategory,
    MissingMetadata,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::InvalidName => write!(f, "Name cannot be empty"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C, D"),
            ValidationError::MissingMetadata => write!(f, "Required metadata fields are missing"),
        }
    }
}

impl Error for ValidationError {}

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

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::InvalidName);
        }
        
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        let valid_categories = ["A", "B", "C", "D"];
        if !valid_categories.contains(&self.category.as_str()) {
            return Err(ValidationError::InvalidCategory);
        }
        
        if self.metadata.is_empty() {
            return Err(ValidationError::MissingMetadata);
        }
        
        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn transform_value(&mut self, multiplier: f64) -> Result<(), Box<dyn Error>> {
        if multiplier <= 0.0 {
            return Err("Multiplier must be positive".into());
        }
        
        self.value *= multiplier;
        Ok(())
    }

    pub fn normalize(&self) -> f64 {
        self.value / 1000.0
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<f64>, Box<dyn Error>> {
    let mut results = Vec::new();
    
    for record in records.iter_mut() {
        record.validate()?;
        
        record.add_metadata("processed".to_string(), "true".to_string());
        record.add_metadata("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
        
        record.transform_value(1.5)?;
        
        results.push(record.normalize());
    }
    
    Ok(results)
}

pub fn filter_records_by_category(records: &[DataRecord], category: &str) -> Vec<&DataRecord> {
    records.iter()
        .filter(|record| record.category == category)
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test");
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "A");
    }

    #[test]
    fn test_validation_success() {
        let mut record = DataRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        record.add_metadata("key".to_string(), "value".to_string());
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let record = DataRecord::new(0, "".to_string(), -10.0, "X".to_string());
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        record.transform_value(2.0).unwrap();
        assert_eq!(record.value, 200.0);
    }

    #[test]
    fn test_normalization() {
        let record = DataRecord::new(1, "Test".to_string(), 500.0, "A".to_string());
        assert_eq!(record.normalize(), 0.5);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            DataRecord::new(1, "A1".to_string(), 100.0, "A".to_string()),
            DataRecord::new(2, "B1".to_string(), 200.0, "B".to_string()),
            DataRecord::new(3, "A2".to_string(), 300.0, "A".to_string()),
        ];
        
        let filtered = filter_records_by_category(&records, "A");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, "Test1".to_string(), 100.0, "A".to_string()),
            DataRecord::new(2, "Test2".to_string(), 200.0, "A".to_string()),
            DataRecord::new(3, "Test3".to_string(), 300.0, "A".to_string()),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 200.0);
        assert_eq!(variance, 6666.666666666667);
        assert_eq!(std_dev, 81.64965809277261);
    }
}
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        DataRecord {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        
        if self.values.is_empty() {
            return Err("Empty values array".into());
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".into());
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) {
        if let Some(max) = self.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            if *max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();
    
    for record in records.iter_mut() {
        record.validate()?;
        record.normalize();
        processed.push(record.clone());
    }
    
    Ok(processed)
}

pub fn aggregate_statistics(records: &[DataRecord]) -> HashMap<u32, (f64, f64, f64)> {
    let mut stats = HashMap::new();
    
    for record in records {
        let record_stats = record.calculate_statistics();
        stats.insert(record.id, record_stats);
    }
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![2.0, 4.0, 6.0]);
        record.normalize();
        
        let expected = vec![1.0/3.0, 2.0/3.0, 1.0];
        for (actual, expected) in record.values.iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0]);
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert!((mean - 2.5).abs() < 1e-10);
        assert!((variance - 1.25).abs() < 1e-10);
        assert!((std_dev - 1.118033988749895).abs() < 1e-10);
    }
}