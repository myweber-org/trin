
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
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
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
            
            self.records.push(DataRecord {
                id,
                value,
                category,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.record_count(), 3);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 15.5).abs() < 0.1);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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
        Ok(DataRecord {
            id,
            value,
            category: category.to_string(),
        })
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
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
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            match DataRecord::new(id, value, parts[2]) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(_) => continue,
            }
        }

        Ok(count)
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,type_a").unwrap();
        writeln!(temp_file, "2,20.0,type_b").unwrap();
        writeln!(temp_file, "3,15.5,type_a").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.get_record_count(), 3);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.333).abs() < 0.001);

        let filtered = processor.filter_by_category("type_a");
        assert_eq!(filtered.len(), 2);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold must be between 0.0 and 1.0, got {}", threshold),
            });
        }
        Ok(Self { threshold })
    }

    pub fn process_data(&self, input: Vec<f64>) -> Result<Vec<f64>, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }

        let normalized: Vec<f64> = input
            .iter()
            .map(|&value| {
                if value.is_nan() || value.is_infinite() {
                    0.0
                } else {
                    value.max(0.0).min(1.0)
                }
            })
            .collect();

        let filtered: Vec<f64> = normalized
            .into_iter()
            .filter(|&value| value >= self.threshold)
            .collect();

        if filtered.is_empty() {
            return Err(ValidationError {
                message: "No data points meet the threshold criteria".to_string(),
            });
        }

        Ok(filtered)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64), ValidationError> {
        if data.is_empty() {
            return Err(ValidationError {
                message: "Cannot calculate statistics for empty dataset".to_string(),
            });
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;

        let variance: f64 = data
            .iter()
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / data.len() as f64;

        Ok((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }

    #[test]
    fn test_data_processing() {
        let processor = DataProcessor::new(0.3).unwrap();
        let input = vec![0.1, 0.4, 0.2, 0.8, 0.5];
        let result = processor.process_data(input);
        assert!(result.is_ok());
        let filtered = result.unwrap();
        assert_eq!(filtered, vec![0.4, 0.8, 0.5]);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.0).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&data);
        assert!(stats.is_ok());
        let (mean, std_dev) = stats.unwrap();
        assert_eq!(mean, 3.0);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
    }
}