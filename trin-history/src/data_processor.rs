
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
    DuplicateTag,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidId => write!(f, "ID must be greater than zero"),
            ProcessingError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ProcessingError::EmptyName => write!(f, "Name cannot be empty"),
            ProcessingError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    statistics: ProcessingStats,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_records: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            statistics: ProcessingStats::default(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(ProcessingError::InvalidId);
        }
        
        self.records.insert(record.id, record.clone());
        self.update_statistics(&record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        let record = self.records.remove(&id);
        if let Some(ref rec) = record {
            self.recalculate_statistics();
        }
        record
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
        self.recalculate_statistics();
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn get_statistics(&self) -> &ProcessingStats {
        &self.statistics
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ProcessingError::EmptyName);
        }
        
        if !(0.0..=1000.0).contains(&record.value) {
            return Err(ProcessingError::InvalidValue);
        }
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &record.tags {
            if !seen_tags.insert(tag) {
                return Err(ProcessingError::DuplicateTag);
            }
        }
        
        Ok(())
    }

    fn update_statistics(&mut self, record: &DataRecord) {
        self.statistics.total_records += 1;
        self.statistics.total_value += record.value;
        self.statistics.average_value = self.statistics.total_value / self.statistics.total_records as f64;
    }

    fn recalculate_statistics(&mut self) {
        self.statistics = ProcessingStats::default();
        for record in self.records.values() {
            self.update_statistics(record);
        }
    }
}

pub fn create_sample_record() -> DataRecord {
    DataRecord {
        id: 1,
        name: String::from("Sample Data"),
        value: 42.5,
        tags: vec![String::from("sample"), String::from("test")],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = create_sample_record();
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let invalid_record = DataRecord {
            id: 0,
            name: String::from("Invalid"),
            value: 50.0,
            tags: vec![],
        };
        
        assert!(processor.add_record(invalid_record).is_err());
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: String::from("Test"),
            value: 10.0,
            tags: vec![],
        };
        
        processor.add_record(record).unwrap();
        processor.transform_values(|v| v * 2.0);
        
        let updated = processor.get_record(1).unwrap();
        assert_eq!(updated.value, 20.0);
    }

    #[test]
    fn test_filter_by_tag() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: String::from("Record 1"),
            value: 10.0,
            tags: vec![String::from("important")],
        };
        
        let record2 = DataRecord {
            id: 2,
            name: String::from("Record 2"),
            value: 20.0,
            tags: vec![String::from("normal")],
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        let important_records = processor.filter_by_tag("important");
        assert_eq!(important_records.len(), 1);
    }
}