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
pub enum ProcessingError {
    InvalidId,
    InvalidValue,
    EmptyName,
    DuplicateRecord,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessingError::InvalidId => write!(f, "ID must be greater than zero"),
            ProcessingError::InvalidValue => write!(f, "Value must be positive"),
            ProcessingError::EmptyName => write!(f, "Name cannot be empty"),
            ProcessingError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    total_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            total_value: 0.0,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::InvalidId);
        }
        
        if record.value <= 0.0 {
            return Err(ProcessingError::InvalidValue);
        }
        
        if record.name.trim().is_empty() {
            return Err(ProcessingError::EmptyName);
        }
        
        if self.records.contains_key(&record.id) {
            return Err(ProcessingError::DuplicateRecord);
        }
        
        self.total_value += record.value;
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        if let Some(record) = self.records.remove(&id) {
            self.total_value -= record.value;
            Some(record)
        } else {
            None
        }
    }

    pub fn total_value(&self) -> f64 {
        self.total_value
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value / self.records.len() as f64)
        }
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        let mut new_total = 0.0;
        
        for record in self.records.values_mut() {
            let new_value = transform_fn(record.value);
            record.value = new_value;
            new_total += new_value;
        }
        
        self.total_value = new_total;
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec!["sample".to_string()],
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.record_count(), 1);
        assert_eq!(processor.total_value(), 100.0);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec![],
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_filter_by_tag() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 50.0,
            tags: vec!["important".to_string(), "test".to_string()],
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Second".to_string(),
            value: 75.0,
            tags: vec!["test".to_string()],
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        let filtered = processor.filter_by_tag("important");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "First");
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec![],
        };
        
        processor.add_record(record).unwrap();
        processor.transform_values(|x| x * 2.0);
        
        assert_eq!(processor.total_value(), 200.0);
        assert_eq!(processor.get_record(1).unwrap().value, 200.0);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    InvalidCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_stats: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if record.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let valid_categories = ["A", "B", "C"];
        if !valid_categories.contains(&record.category.as_str()) {
            return Err(ValidationError::InvalidCategory);
        }
        
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        Self::validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(format!("Record with ID {} already exists", record.id).into());
        }
        
        let category_total = self.category_stats
            .entry(record.category.clone())
            .or_insert(0.0);
        *category_total += record.value;
        
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.values().map(|r| r.value).sum()
    }

    pub fn get_category_average(&self, category: &str) -> Option<f64> {
        let count = self.records.values()
            .filter(|r| r.category == category)
            .count();
        
        if count == 0 {
            return None;
        }
        
        let total = self.category_stats.get(category)?;
        Some(*total / count as f64)
    }

    pub fn transform_records<F>(&self, transform_fn: F) -> Vec<DataRecord>
    where
        F: Fn(&DataRecord) -> DataRecord,
    {
        self.records.values()
            .map(|record| transform_fn(record))
            .collect()
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<&DataRecord>
    where
        F: Fn(&DataRecord) -> bool,
    {
        self.records.values()
            .filter(|record| predicate(record))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(DataProcessor::validate_record(&valid_record).is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "D".to_string(),
        };
        
        assert!(DataProcessor::validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Item1".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Item2".to_string(),
            value: 75.0,
            category: "A".to_string(),
        };
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        assert_eq!(processor.calculate_total_value(), 125.0);
        assert_eq!(processor.get_category_average("A"), Some(62.5));
    }
}