
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of valid range", val),
            DataError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if values.is_empty() {
            return Err(DataError::EmptyValues);
        }
        
        for &value in &values {
            if !value.is_finite() {
                return Err(DataError::ValueOutOfRange(value));
            }
        }
        
        Ok(Self {
            id,
            values,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    pub fn validate_required_metadata(&self, required_keys: &[&str]) -> Result<(), DataError> {
        for key in required_keys {
            if !self.metadata.contains_key(*key) {
                return Err(DataError::MissingMetadata(key.to_string()));
            }
        }
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
    
    pub fn normalize_values(&mut self) {
        let (mean, _, std_dev) = self.calculate_statistics();
        
        if std_dev > 0.0 {
            for value in &mut self.values {
                *value = (*value - mean) / std_dev;
            }
        }
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<(u32, f64)>, DataError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate_required_metadata(&["source", "timestamp"])?;
        record.normalize_values();
        
        let (mean, _, _) = record.calculate_statistics();
        results.push((record.id, mean));
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.values, vec![1.0, 2.0, 3.0]);
    }
    
    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(matches!(result, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_empty_values() {
        let result = DataRecord::new(1, vec![]);
        assert!(matches!(result, Err(DataError::EmptyValues)));
    }
    
    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0]).unwrap();
        record.add_metadata("source".to_string(), "sensor_a".to_string());
        
        assert_eq!(record.get_metadata("source"), Some(&"sensor_a".to_string()));
        assert_eq!(record.get_metadata("nonexistent"), None);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert!((mean - 3.0).abs() < 1e-10);
        assert!((variance - 2.0).abs() < 1e-10);
        assert!((std_dev - 2.0_f64.sqrt()).abs() < 1e-10);
    }
    
    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        record.normalize_values();
        
        let (mean, _, std_dev) = record.calculate_statistics();
        assert!(mean.abs() < 1e-10);
        assert!((std_dev - 1.0).abs() < 1e-10);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<Statistics> {
        if records.is_empty() || column_index >= records[0].len() {
            return None;
        }

        let mut values = Vec::new();
        for record in records {
            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
            }
        }

        if values.is_empty() {
            return None;
        }

        Some(Statistics::from_values(&values))
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

impl Statistics {
    fn from_values(values: &[f64]) -> Self {
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Statistics {
            count,
            mean,
            min,
            max,
            sum,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,88.0").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&records, 0).unwrap();
        
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 12.666666666666666);
        assert_eq!(stats.min, 10.5);
        assert_eq!(stats.max, 15.5);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
    }
}

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let value = parts[1].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let category = parts[2].trim();

        match DataRecord::new(id, value, category) {
            Ok(record) => records.push(record),
            Err(e) => return Err(format!("Error at line {}: {}", line_number, e).into()),
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, "category_a").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "category_a");
    }

    #[test]
    fn test_invalid_record_negative_value() {
        let result = DataRecord::new(1, -5.0, "category_a");
        assert!(result.is_err());
    }

    #[test]
    fn test_process_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,42.5,category_a").unwrap();
        writeln!(temp_file, "2,100.0,category_b").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,75.3,category_c").unwrap();

        let records = process_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].value, 100.0);
        assert_eq!(records[2].category, "category_c");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, "cat1").unwrap(),
            DataRecord::new(2, 20.0, "cat2").unwrap(),
            DataRecord::new(3, 30.0, "cat3").unwrap(),
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_empty_statistics() {
        let records: Vec<DataRecord> = vec![];
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 0.0);
        assert_eq!(variance, 0.0);
        assert_eq!(std_dev, 0.0);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error")]
    TransformationError,
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
    transformation_pipeline: Vec<TransformationStep>,
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

    pub fn add_transformation_step(&mut self, step: TransformationStep) {
        self.transformation_pipeline.push(step);
    }

    pub fn process(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate(record)?;
        self.transform(record)
    }

    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            rule.apply(record)?;
        }
        Ok(())
    }

    fn transform(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        for step in &self.transformation_pipeline {
            transformed = step.apply(&transformed)?;
        }
        
        Ok(transformed)
    }
}

pub trait ValidationRule {
    fn apply(&self, record: &DataRecord) -> Result<(), ProcessingError>;
}

pub trait TransformationStep {
    fn apply(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError>;
}

pub struct TimestampValidator {
    min_timestamp: i64,
    max_timestamp: i64,
}

impl TimestampValidator {
    pub fn new(min: i64, max: i64) -> Self {
        TimestampValidator {
            min_timestamp: min,
            max_timestamp: max,
        }
    }
}

impl ValidationRule for TimestampValidator {
    fn apply(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.timestamp < self.min_timestamp || record.timestamp > self.max_timestamp {
            return Err(ProcessingError::ValidationFailed(
                format!("Timestamp {} out of range [{}, {}]", 
                    record.timestamp, self.min_timestamp, self.max_timestamp)
            ));
        }
        Ok(())
    }
}

pub struct ValueNormalizer {
    target_range: (f64, f64),
}

impl ValueNormalizer {
    pub fn new(min: f64, max: f64) -> Self {
        ValueNormalizer {
            target_range: (min, max),
        }
    }
}

impl TransformationStep for ValueNormalizer {
    fn apply(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut normalized = record.clone();
        
        for (key, value) in normalized.values.iter_mut() {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::TransformationError);
            }
            
            *value = (*value - self.target_range.0) / (self.target_range.1 - self.target_range.0);
        }
        
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_validation() {
        let validator = TimestampValidator::new(1000, 2000);
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1500,
            values: HashMap::new(),
            tags: Vec::new(),
        };
        
        assert!(validator.apply(&valid_record).is_ok());
        
        let invalid_record = DataRecord {
            id: 2,
            timestamp: 500,
            values: HashMap::new(),
            tags: Vec::new(),
        };
        
        assert!(validator.apply(&invalid_record).is_err());
    }

    #[test]
    fn test_value_normalization() {
        let normalizer = ValueNormalizer::new(0.0, 100.0);
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 50.0);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values,
            tags: Vec::new(),
        };
        
        let result = normalizer.apply(&record).unwrap();
        assert_eq!(result.values.get("temperature").unwrap(), &0.5);
    }
}