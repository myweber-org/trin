
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
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
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::EmptyCategory => write!(f, "Category cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if self.category.trim().is_empty() {
            return Err(ValidationError::EmptyCategory);
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.category = self.category.to_uppercase();
    }
}

pub fn process_records(records: &mut Vec<DataRecord>) -> Result<Vec<DataRecord>, ValidationError> {
    let mut processed = Vec::new();
    
    for record in records {
        record.validate()?;
        let mut transformed = record.clone();
        transformed.transform(1.5);
        processed.push(transformed);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 100.0,
            category: "test".to_string(),
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 100.0,
            category: "test".to_string(),
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            value: 100.0,
            category: "example".to_string(),
        };
        
        record.transform(2.0);
        assert_eq!(record.value, 200.0);
        assert_eq!(record.category, "EXAMPLE");
    }
}