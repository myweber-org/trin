
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
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::DuplicateTag => write!(f, "Tags contain duplicates"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        if name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        if value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let mut seen_tags = HashMap::new();
        for tag in &tags {
            if seen_tags.contains_key(tag) {
                return Err(ValidationError::DuplicateTag);
            }
            seen_tags.insert(tag.clone(), true);
        }
        
        Ok(DataRecord {
            id,
            name,
            value,
            tags,
        })
    }
    
    pub fn transform(&self, multiplier: f64) -> Self {
        DataRecord {
            id: self.id,
            name: self.name.clone(),
            value: self.value * multiplier,
            tags: self.tags.clone(),
        }
    }
    
    pub fn add_tag(&mut self, tag: String) -> bool {
        if self.tags.contains(&tag) {
            return false;
        }
        self.tags.push(tag);
        true
    }
    
    pub fn calculate_score(&self) -> f64 {
        let tag_bonus = self.tags.len() as f64 * 0.5;
        self.value + tag_bonus
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|r| r.value > 10.0)
        .map(|r| r.transform(1.1))
        .collect()
}

pub fn aggregate_values(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut result = HashMap::new();
    
    for record in records {
        let entry = result.entry(record.name.clone()).or_insert(0.0);
        *entry += record.value;
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test".to_string(),
            15.5,
            vec!["tag1".to_string(), "tag2".to_string()]
        ).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test");
        assert_eq!(record.value, 15.5);
        assert_eq!(record.tags.len(), 2);
    }
    
    #[test]
    fn test_invalid_record_creation() {
        let result = DataRecord::new(
            0,
            "Test".to_string(),
            15.5,
            vec!["tag1".to_string()]
        );
        assert!(result.is_err());
        
        let result = DataRecord::new(
            1,
            "".to_string(),
            15.5,
            vec!["tag1".to_string()]
        );
        assert!(result.is_err());
        
        let result = DataRecord::new(
            1,
            "Test".to_string(),
            -5.0,
            vec!["tag1".to_string()]
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_transform() {
        let record = DataRecord::new(
            1,
            "Test".to_string(),
            10.0,
            vec!["tag1".to_string()]
        ).unwrap();
        
        let transformed = record.transform(2.0);
        assert_eq!(transformed.value, 20.0);
    }
    
    #[test]
    fn test_calculate_score() {
        let record = DataRecord::new(
            1,
            "Test".to_string(),
            10.0,
            vec!["tag1".to_string(), "tag2".to_string()]
        ).unwrap();
        
        assert_eq!(record.calculate_score(), 11.0);
    }
}