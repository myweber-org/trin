
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
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
            ValidationError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
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
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &self.tags {
            if !seen_tags.insert(tag) {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> &mut Self {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
        self.tags.sort();
        self.tags.dedup();
        self
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
        
        if self.records.contains_key(&record.id) {
            return Err(format!("Record with ID {} already exists", record.id).into());
        }
        
        self.records.insert(record.id, record);
        Ok(())
    }
    
    pub fn process_records(&mut self, multiplier: f64) -> Vec<DataRecord> {
        let mut processed = Vec::new();
        
        for (_, record) in &mut self.records {
            let mut transformed = record.clone();
            transformed.transform(multiplier);
            processed.push(transformed);
        }
        
        processed.sort_by(|a, b| a.id.cmp(&b.id));
        processed
    }
    
    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let values: Vec<f64> = self.records.values().map(|r| r.value).collect();
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let max = values.iter().fold(f64::MIN, |a, &b| a.max(b));
        
        (mean, variance.sqrt(), max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -5.0,
            tags: vec!["tag1".to_string(), "tag1".to_string()],
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 10.0,
            tags: vec!["z".to_string(), "a".to_string(), "z".to_string()],
        };
        
        record.transform(2.0);
        
        assert_eq!(record.name, "TEST");
        assert_eq!(record.value, 20.0);
        assert_eq!(record.tags, vec!["a".to_string(), "z".to_string()]);
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 10.0,
            tags: vec!["alpha".to_string()],
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Second".to_string(),
            value: 20.0,
            tags: vec!["beta".to_string()],
        };
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        let processed = processor.process_records(1.5);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].value, 15.0);
        assert_eq!(processed[1].value, 30.0);
        
        let (mean, std_dev, max) = processor.get_statistics();
        assert!((mean - 15.0).abs() < 0.001);
        assert!((std_dev - 7.5).abs() < 0.001);
        assert_eq!(max, 30.0);
    }
}