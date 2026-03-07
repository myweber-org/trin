
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    MissingField,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be positive integer"),
            DataError::InvalidValue => write!(f, "Value must be between 0 and 1000"),
            DataError::MissingField => write!(f, "Required field is missing"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_totals: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_totals: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.value < 0.0 || record.value > 1000.0 {
            return Err(DataError::InvalidValue);
        }

        if record.name.is_empty() || record.category.is_empty() {
            return Err(DataError::MissingField);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        let category_total = self.category_totals
            .entry(record.category.clone())
            .or_insert(0.0);
        *category_total += record.value;

        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_total(&self, category: &str) -> f64 {
        *self.category_totals.get(category).unwrap_or(&0.0)
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let total: f64 = self.records.values().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }

        self.recalculate_totals();
    }

    fn recalculate_totals(&mut self) {
        self.category_totals.clear();
        for record in self.records.values() {
            let category_total = self.category_totals
                .entry(record.category.clone())
                .or_insert(0.0);
            *category_total += record.value;
        }
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.category_totals.clear();
    }
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            category,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn category(&self) -> &str {
        &self.category
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord::new(1, "Test1".to_string(), 100.0, "A".to_string());
        let record2 = DataRecord::new(1, "Test2".to_string(), 200.0, "B".to_string());
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
    }

    #[test]
    fn test_category_total() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, "A1".to_string(), 100.0, "CategoryA".to_string())).unwrap();
        processor.add_record(DataRecord::new(2, "A2".to_string(), 200.0, "CategoryA".to_string())).unwrap();
        processor.add_record(DataRecord::new(3, "B1".to_string(), 150.0, "CategoryB".to_string())).unwrap();
        
        assert_eq!(processor.get_category_total("CategoryA"), 300.0);
        assert_eq!(processor.get_category_total("CategoryB"), 150.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, "Test".to_string(), 100.0, "A".to_string())).unwrap();
        
        processor.transform_values(|x| x * 2.0);
        
        let record = processor.get_record(1).unwrap();
        assert_eq!(record.value(), 200.0);
    }
}