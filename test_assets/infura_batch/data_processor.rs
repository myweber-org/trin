
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::EmptyCategory => write!(f, "Category cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(ValidationError::EmptyCategory);
        }
        
        Ok(Self {
            id,
            value,
            category: category.trim().to_string(),
        })
    }
    
    pub fn transform_value(&mut self, multiplier: f64) -> Result<(), ValidationError> {
        let new_value = self.value * multiplier;
        
        if new_value < 0.0 || new_value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        self.value = new_value;
        Ok(())
    }
    
    pub fn normalize(&self) -> f64 {
        self.value / 1000.0
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<f64>, ValidationError> {
    let mut results = Vec::with_capacity(records.len());
    
    for record in records {
        record.transform_value(1.5)?;
        results.push(record.normalize());
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, "test".to_string());
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 500.0, "test".to_string());
        assert!(matches!(record, Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 200.0, "test".to_string()).unwrap();
        assert!(record.transform_value(2.0).is_ok());
        assert_eq!(record.value, 400.0);
    }
    
    #[test]
    fn test_normalize() {
        let record = DataRecord::new(1, 250.0, "test".to_string()).unwrap();
        assert_eq!(record.normalize(), 0.25);
    }
}