
use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        Self {
            id,
            name,
            value,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".into());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".into());
        }
        if self.id == 0 {
            return Err("ID must be greater than zero".into());
        }
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn transform_value<F>(&mut self, transformer: F)
    where
        F: Fn(f64) -> f64,
    {
        self.value = transformer(self.value);
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = DataRecord::new(
            record.id,
            record.name.to_uppercase(),
            record.value * 1.1,
        );
        
        for tag in &record.tags {
            processed_record.add_tag(tag.clone());
        }
        
        for (key, value) in &record.metadata {
            processed_record.set_metadata(key.clone(), value.clone());
        }
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 100.0);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, "".to_string(), -10.0);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 50.0);
        record.transform_value(|x| x * 2.0);
        assert_eq!(record.value, 100.0);
    }

    #[test]
    fn test_record_processing() {
        let mut records = vec![
            DataRecord::new(1, "alpha".to_string(), 10.0),
            DataRecord::new(2, "beta".to_string(), 20.0),
        ];
        
        let result = process_records(&mut records);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed[0].name, "ALPHA");
        assert_eq!(processed[1].value, 22.0);
    }
}