
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub category: String,
    pub count: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        self.records.push(record.clone());
        self.update_category_stats(&record);
        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string(),
            ));
        }

        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError(
                "Record value cannot be negative".to_string(),
            ));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record category cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                category: record.category.clone(),
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.count += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.count as f64;
    }

    pub fn get_records_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn calculate_overall_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) -> Result<(), ProcessingError>
    where
        F: Fn(f64) -> Result<f64, ProcessingError>,
    {
        for record in &mut self.records {
            match transform_fn(record.value) {
                Ok(new_value) => record.value = new_value,
                Err(e) => return Err(e),
            }
        }
        self.recalculate_stats();
        Ok(())
    }

    fn recalculate_stats(&mut self) {
        self.category_stats.clear();
        for record in &self.records {
            self.update_category_stats(record);
        }
    }

    pub fn get_total_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.category_stats.clear();
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
    #[error("Duplicate record ID: {0}")]
    DuplicateId(u64),
}

pub struct DataProcessor {
    cache: HashMap<u64, DataRecord>,
    validation_rules: ValidationRules,
}

#[derive(Clone)]
pub struct ValidationRules {
    pub max_values: usize,
    pub allowed_tags: Vec<String>,
    pub timestamp_range: (i64, i64),
}

impl DataProcessor {
    pub fn new(rules: ValidationRules) -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn process_record(&mut self, record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        
        if self.cache.contains_key(&record.id) {
            return Err(ProcessingError::DuplicateId(record.id));
        }

        let processed = self.transform_record(record.clone());
        self.cache.insert(processed.id, processed.clone());
        
        Ok(processed)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.validation_rules.max_values {
            return Err(ProcessingError::OutOfRange(
                format!("Too many values: {}", record.values.len())
            ));
        }

        if record.timestamp < self.validation_rules.timestamp_range.0 
            || record.timestamp > self.validation_rules.timestamp_range.1 {
            return Err(ProcessingError::OutOfRange(
                format!("Timestamp out of range: {}", record.timestamp)
            ));
        }

        for tag in &record.tags {
            if !self.validation_rules.allowed_tags.contains(tag) {
                return Err(ProcessingError::InvalidFormat);
            }
        }

        Ok(())
    }

    fn transform_record(&self, mut record: DataRecord) -> DataRecord {
        let normalized_values: HashMap<String, f64> = record.values
            .iter()
            .map(|(k, v)| (k.to_lowercase(), *v))
            .collect();
        
        record.values = normalized_values;
        record.tags.sort();
        record.tags.dedup();
        
        record
    }

    pub fn get_record(&self, id: u64) -> Option<&DataRecord> {
        self.cache.get(&id)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rules() -> ValidationRules {
        ValidationRules {
            max_values: 10,
            allowed_tags: vec!["production".to_string(), "test".to_string()],
            timestamp_range: (0, 1000000000),
        }
    }

    #[test]
    fn test_valid_record_processing() {
        let mut processor = DataProcessor::new(create_test_rules());
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 23.5);
        
        let record = DataRecord {
            id: 1,
            timestamp: 123456789,
            values,
            tags: vec!["production".to_string()],
        };

        let result = processor.process_record(record);
        assert!(result.is_ok());
        assert_eq!(processor.cache_size(), 1);
    }

    #[test]
    fn test_duplicate_id() {
        let mut processor = DataProcessor::new(create_test_rules());
        let record1 = DataRecord {
            id: 1,
            timestamp: 100,
            values: HashMap::new(),
            tags: vec![],
        };
        
        let record2 = DataRecord {
            id: 1,
            timestamp: 200,
            values: HashMap::new(),
            tags: vec![],
        };

        let _ = processor.process_record(record1);
        let result = processor.process_record(record2);
        
        assert!(matches!(result, Err(ProcessingError::DuplicateId(1))));
    }
}