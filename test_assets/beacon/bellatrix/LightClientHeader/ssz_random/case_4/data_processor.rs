
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
            ValidationError::DuplicateTag => write!(f, "Tags contain duplicates"),
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
    
    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    statistics: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            statistics: HashMap::new(),
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ValidationError> {
        record.validate()?;
        self.records.push(record);
        self.update_statistics();
        Ok(())
    }
    
    pub fn process_all(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform(multiplier);
        }
        self.update_statistics();
    }
    
    fn update_statistics(&mut self) {
        self.statistics.clear();
        
        if self.records.is_empty() {
            return;
        }
        
        let count = self.records.len() as f64;
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let avg = sum / count;
        let max = self.records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
        let min = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);
        
        self.statistics.insert("count".to_string(), count);
        self.statistics.insert("sum".to_string(), sum);
        self.statistics.insert("average".to_string(), avg);
        self.statistics.insert("maximum".to_string(), max);
        self.statistics.insert("minimum".to_string(), min);
    }
    
    pub fn get_statistics(&self) -> &HashMap<String, f64> {
        &self.statistics
    }
    
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec![],
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "sample".to_string(),
            value: 50.0,
            tags: vec!["test".to_string()],
        };
        
        assert!(processor.add_record(record).is_ok());
        processor.process_all(2.0);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.get("count"), Some(&1.0));
        assert_eq!(stats.get("sum"), Some(&100.0));
    }
}