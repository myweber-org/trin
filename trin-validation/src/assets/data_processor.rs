
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
    MissingTags,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::MissingTags => write!(f, "At least one tag is required"),
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
        if self.tags.is_empty() {
            return Err(ValidationError::MissingTags);
        }
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.tags.sort();
        self.tags.dedup();
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        record.validate()?;
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn process_records(&mut self, multiplier: f64) {
        for record in self.records.values_mut() {
            record.transform(multiplier);
        }
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let total_value: f64 = self.records.values().map(|r| r.value).sum();
        let avg_value = if count > 0 {
            total_value / count as f64
        } else {
            0.0
        };
        (count, total_value, avg_value)
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.tags.iter().any(|t| t == tag))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            42.5,
            vec!["tag1".to_string(), "tag2".to_string()],
        );
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "Test".to_string(),
            10.0,
            vec!["tag".to_string()],
        );
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(
            1,
            "Sample".to_string(),
            100.0,
            vec!["important".to_string(), "test".to_string()],
        );
        
        assert!(processor.add_record(record).is_ok());
        processor.process_records(2.0);
        
        let (count, total, avg) = processor.get_statistics();
        assert_eq!(count, 1);
        assert_eq!(total, 200.0);
        assert_eq!(avg, 200.0);
        
        let tagged = processor.find_by_tag("important");
        assert_eq!(tagged.len(), 1);
    }
}