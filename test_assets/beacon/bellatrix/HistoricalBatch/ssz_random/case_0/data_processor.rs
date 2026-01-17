
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();
            let record = DataRecord::new(id, value, category);
            
            self.total_value += value;
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        Some(self.total_value / self.records.len() as f64)
    }

    pub fn get_category_summary(&self) -> Vec<(String, f64)> {
        let mut summary = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                let entry = summary.entry(record.category.clone()).or_insert(0.0);
                *entry += record.value;
            }
        }

        let mut result: Vec<(String, f64)> = summary.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,15.3,TypeB").unwrap();
        writeln!(temp_file, "3,8.7,TypeA").unwrap();
        
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        
        let valid_records = processor.filter_valid_records();
        assert_eq!(valid_records.len(), 3);
        
        let average = processor.calculate_average().unwrap();
        assert!((average - 11.5).abs() < 0.001);
        
        let summary = processor.get_category_summary();
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].0, "TypeA");
        assert!((summary[0].1 - 19.2).abs() < 0.001);
    }
}use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = Self::validate_data(data)?;
        let normalized = Self::normalize_data(&validated);
        let transformed = Self::apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(data: &[f64]) -> Result<Vec<f64>, String> {
        if data.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }
        Ok(data.to_vec())
    }

    fn normalize_data(data: &[f64]) -> Vec<f64> {
        let max_value = data.iter().fold(f64::MIN, |a, &b| a.max(b));
        if max_value.abs() < f64::EPSILON {
            return data.to_vec();
        }
        data.iter().map(|&x| x / max_value).collect()
    }

    fn apply_transformations(data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| (x * 100.0).round() / 100.0)
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0];
        
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        assert!(processed[3] <= 1.0);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_numeric_data("invalid", &data);
        assert!(result.is_err());
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

pub struct DataProcessor {
    validation_rules: Vec<ValidationRule>,
    transformation_pipeline: Vec<Transformation>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformation_pipeline.push(transformation);
    }

    pub fn process(&self, record: &mut DataRecord) -> Result<(), DataError> {
        for rule in &self.validation_rules {
            rule.validate(record)?;
        }

        for transformation in &self.transformation_pipeline {
            transformation.apply(record);
        }

        Ok(())
    }

    pub fn batch_process(&self, records: &mut [DataRecord]) -> Vec<Result<(), DataError>> {
        records
            .iter_mut()
            .map(|record| self.process(record))
            .collect()
    }
}

pub trait ValidationRule {
    fn validate(&self, record: &DataRecord) -> Result<(), DataError>;
}

pub struct RequiredFieldRule {
    field_name: String,
}

impl RequiredFieldRule {
    pub fn new(field_name: &str) -> Self {
        RequiredFieldRule {
            field_name: field_name.to_string(),
        }
    }
}

impl ValidationRule for RequiredFieldRule {
    fn validate(&self, record: &DataRecord) -> Result<(), DataError> {
        if !record.values.contains_key(&self.field_name) {
            return Err(DataError::MissingField(self.field_name.clone()));
        }
        Ok(())
    }
}

pub trait Transformation {
    fn apply(&self, record: &mut DataRecord);
}

pub struct NormalizeTransformation {
    field_name: String,
    target_mean: f64,
    target_std: f64,
}

impl NormalizeTransformation {
    pub fn new(field_name: &str, target_mean: f64, target_std: f64) -> Self {
        NormalizeTransformation {
            field_name: field_name.to_string(),
            target_mean,
            target_std,
        }
    }
}

impl Transformation for NormalizeTransformation {
    fn apply(&self, record: &mut DataRecord) {
        if let Some(value) = record.values.get_mut(&self.field_name) {
            *value = (*value - self.target_mean) / self.target_std;
        }
    }
}

pub struct AddTimestampTagTransformation;

impl Transformation for AddTimestampTagTransformation {
    fn apply(&self, record: &mut DataRecord) {
        let hour_tag = format!("hour_{}", record.timestamp / 3600);
        if !record.tags.contains(&hour_tag) {
            record.tags.push(hour_tag);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_field_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::new(),
            tags: Vec::new(),
        };

        let rule = RequiredFieldRule::new("temperature");
        assert!(rule.validate(&record).is_err());

        record.values.insert("temperature".to_string(), 25.5);
        assert!(rule.validate(&record).is_ok());
    }

    #[test]
    fn test_normalize_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("value".to_string(), 150.0)]),
            tags: Vec::new(),
        };

        let transform = NormalizeTransformation::new("value", 100.0, 50.0);
        transform.apply(&mut record);

        assert_eq!(record.values.get("value"), Some(&1.0));
    }

    #[test]
    fn test_processor_pipeline() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(RequiredFieldRule::new("pressure"));
        processor.add_transformation(AddTimestampTagTransformation);

        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("pressure".to_string(), 1013.25)]),
            tags: Vec::new(),
        };

        let result = processor.process(&mut record);
        assert!(result.is_ok());
        assert!(record.tags.contains(&"hour_342935".to_string()));
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
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyName,
    UnknownCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::UnknownCategory => write!(f, "Category not recognized"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(allowed_categories: Vec<String>) -> Self {
        DataProcessor {
            records: HashMap::new(),
            categories: allowed_categories,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }
        
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        self.records.remove(&id)
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
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
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        
        if !self.categories.contains(&record.category) {
            return Err(DataError::UnknownCategory);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let categories = vec!["A".to_string(), "B".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_invalid_record() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "B".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_calculate_average() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 50.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 100.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 150.0, category: "A".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.calculate_average(), 100.0);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::EmptyCategory => write!(f, "Category cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(ValidationError::EmptyCategory);
        }
        
        Ok(Self {
            id,
            value,
            category: category.trim().to_string(),
        })
    }
    
    pub fn transform_value(&mut self, multiplier: f64) -> Result<(), ValidationError> {
        let new_value = self.value * multiplier;
        
        if new_value < 0.0 || new_value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        self.value = new_value;
        Ok(())
    }
    
    pub fn normalize(&self) -> f64 {
        self.value / 1000.0
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<f64>, ValidationError> {
    let mut results = Vec::with_capacity(records.len());
    
    for record in records {
        record.transform_value(1.5)?;
        results.push(record.normalize());
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, "test".to_string());
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 500.0, "test".to_string());
        assert!(matches!(record, Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_value() {
        let mut record = DataRecord::new(1, 200.0, "test".to_string()).unwrap();
        assert!(record.transform_value(2.0).is_ok());
        assert_eq!(record.value, 400.0);
    }
    
    #[test]
    fn test_normalize() {
        let record = DataRecord::new(1, 250.0, "test".to_string()).unwrap();
        assert_eq!(record.normalize(), 0.25);
    }
}