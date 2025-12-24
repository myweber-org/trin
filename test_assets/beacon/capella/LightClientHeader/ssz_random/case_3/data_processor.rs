
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