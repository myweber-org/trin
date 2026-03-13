
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct DataProcessor {
    buffer_size: usize,
}

impl DataProcessor {
    pub fn new(buffer_size: usize) -> Self {
        DataProcessor { buffer_size }
    }

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::with_capacity(self.buffer_size, file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let mut records = Vec::new();
        for line in content.lines() {
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), &'static str> {
        if records.is_empty() {
            return Err("No valid records found");
        }

        let expected_columns = records[0].len();
        for (index, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                return Err(&format!("Record {} has {} columns, expected {}", 
                    index, record.len(), expected_columns));
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64), &'static str> {
        let mut values = Vec::new();
        
        for record in records.iter().skip(1) {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return Err("No numeric values found in specified column");
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();

        Ok((mean, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let processor = DataProcessor::new(1024);
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        
        let result = processor.process_csv(temp_file.path());
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[1][0], "Alice");
    }

    #[test]
    fn test_validate_records() {
        let processor = DataProcessor::new(1024);
        let valid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let invalid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string()],
        ];
        
        assert!(processor.validate_records(&valid_records).is_ok());
        assert!(processor.validate_records(&invalid_records).is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(1024);
        let records = vec![
            vec!["name".to_string(), "value".to_string()],
            vec!["test1".to_string(), "10.0".to_string()],
            vec!["test2".to_string(), "20.0".to_string()],
            vec!["test3".to_string(), "30.0".to_string()],
        ];
        
        let result = processor.calculate_statistics(&records, 1);
        assert!(result.is_ok());
        
        let (mean, std_dev) = result.unwrap();
        assert_eq!(mean, 20.0);
        assert!(std_dev > 0.0);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        for &value in data {
            if !value.is_finite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }
        Ok(data.to_vec())
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).ln_1p().tanh())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_rejects_invalid_data() {
        let processor = DataProcessor::new();
        let data = vec![1.0, f64::NAN, 3.0];
        assert!(processor.validate_data(&data).is_err());
    }

    #[test]
    fn test_normalization_produces_zero_mean() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = processor.normalize_data(&data);
        let mean = normalized.iter().sum::<f64>() / normalized.len() as f64;
        assert!(mean.abs() < 1e-10);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![1.5, 2.5, 3.5];
        
        let result1 = processor.process_dataset("test", &data).unwrap();
        let result2 = processor.process_dataset("test", &data).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(processor.cache_stats(), (1, 3));
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
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn transform(&self) -> f64 {
        if self.valid {
            self.value * 2.5
        } else {
            0.0
        }
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

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].trim();

            let record = DataRecord::new(id, value, category);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.filter_valid()
            .iter()
            .map(|r| r.transform())
            .sum()
    }

    pub fn get_statistics(&self) -> (usize, usize, f64) {
        let valid_count = self.filter_valid().len();
        let total_count = self.records.len();
        let total_value = self.calculate_total();
        
        (valid_count, total_count, total_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 42.5, "category_a");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "category_a");
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "category_b");
        assert!(!record.is_valid());
    }

    #[test]
    fn test_transform() {
        let valid_record = DataRecord::new(3, 10.0, "test");
        assert_eq!(valid_record.transform(), 25.0);
        
        let invalid_record = DataRecord::new(4, -10.0, "test");
        assert_eq!(invalid_record.transform(), 0.0);
    }

    #[test]
    fn test_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, 10.0, "a");
        let record2 = DataRecord::new(2, -5.0, "b");
        
        processor.records.push(record1);
        processor.records.push(record2);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 1);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 1);
        assert_eq!(stats.1, 2);
        assert_eq!(stats.2, 25.0);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Processing timeout")]
    Timeout,
    #[error("Serialization error")]
    Serialization,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidInput("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::InvalidInput("Timestamp cannot be negative".to_string()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::InvalidInput("Value key cannot be empty".to_string()));
            }
            if !value.is_finite() {
                return Err(DataError::InvalidInput(format!("Value {} is not finite", key)));
            }
        }
        
        Ok(())
    }
    
    pub fn normalize_values(&mut self) {
        let sum: f64 = self.values.values().sum();
        if sum != 0.0 {
            for value in self.values.values_mut() {
                *value /= sum;
            }
        }
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

pub struct DataProcessor {
    max_records: usize,
    processed_count: usize,
}

impl DataProcessor {
    pub fn new(max_records: usize) -> Self {
        DataProcessor {
            max_records,
            processed_count: 0,
        }
    }
    
    pub fn process_batch(&mut self, records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
        if self.processed_count + records.len() > self.max_records {
            return Err(DataError::InvalidInput("Exceeds maximum record limit".to_string()));
        }
        
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records.iter_mut() {
            record.validate()?;
            record.normalize_values();
            record.add_tag("processed".to_string());
            processed.push(record.clone());
        }
        
        self.processed_count += records.len();
        Ok(processed)
    }
    
    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("processed_count".to_string(), self.processed_count);
        stats.insert("remaining_capacity".to_string(), self.max_records - self.processed_count);
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("temp".to_string(), 25.5)]),
            tags: vec![],
        };
        
        assert!(record.validate().is_ok());
        
        record.id = 0;
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_value_normalization() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("a".to_string(), 1.0),
                ("b".to_string(), 2.0),
                ("c".to_string(), 3.0),
            ]),
            tags: vec![],
        };
        
        record.normalize_values();
        
        let sum: f64 = record.values.values().sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
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
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidName,
    InvalidValue,
    EmptyTags,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidName => write!(f, "Name cannot be empty"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyTags => write!(f, "Record must have at least one tag"),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if name.trim().is_empty() {
            return Err(DataError::InvalidName);
        }
        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        if tags.is_empty() {
            return Err(DataError::EmptyTags);
        }

        Ok(Self {
            id,
            name,
            value,
            tags,
        })
    }

    pub fn transform(&self, multiplier: f64) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            value: self.value * multiplier,
            tags: self.tags.clone(),
        }
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if self.records.contains_key(&record.id) {
            return Err(DataError::InvalidId);
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

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.has_tag(tag))
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn transform_all(&mut self, multiplier: f64) {
        for record in self.records.values_mut() {
            *record = record.transform(multiplier);
        }
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag2".to_string()],
        );
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "Test".to_string(),
            100.0,
            vec!["tag".to_string()],
        );
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(
            1,
            "Record 1".to_string(),
            50.0,
            vec!["important".to_string()],
        ).unwrap();
        
        let record2 = DataRecord::new(
            2,
            "Record 2".to_string(),
            150.0,
            vec!["important".to_string(), "urgent".to_string()],
        ).unwrap();

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        assert_eq!(processor.count_records(), 2);
        assert_eq!(processor.filter_by_tag("important").len(), 2);
        assert_eq!(processor.filter_by_tag("urgent").len(), 1);
        
        processor.transform_all(2.0);
        assert_eq!(processor.get_record(1).unwrap().value, 100.0);
        
        processor.clear();
        assert_eq!(processor.count_records(), 0);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_data(&mut self, key: &str, values: Vec<f64>) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data set provided".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }

        let processed: Vec<f64> = values
            .iter()
            .map(|&x| x * 2.0 - 1.0)
            .filter(|&x| x > 0.0)
            .collect();

        if processed.is_empty() {
            return Err("No valid data after processing".to_string());
        }

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn get_cached_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|data| {
            let sum: f64 = data.iter().sum();
            let mean = sum / data.len() as f64;
            let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
            let std_dev = variance.sqrt();
            (mean, variance, std_dev)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_valid_data() {
        let mut processor = DataProcessor::new();
        let result = processor.process_data("test1", vec![1.0, 2.0, 3.0]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1.0, 3.0, 5.0]);
    }

    #[test]
    fn test_process_invalid_data() {
        let mut processor = DataProcessor::new();
        let result = processor.process_data("test2", vec![f64::NAN, 2.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.process_data("stats", vec![1.0, 2.0, 3.0]).unwrap();
        let stats = processor.calculate_statistics("stats").unwrap();
        assert!((stats.0 - 3.0).abs() < 0.001);
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

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        for &value in data {
            if !value.is_finite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }
        Ok(data.to_vec())
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).ln_1p())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_entries = self.cache.len();
        let total_values = self.cache.values()
            .map(|v| v.len())
            .sum();
        (total_entries, total_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let processor = DataProcessor::new();
        let valid_data = vec![1.0, 2.0, 3.0];
        let invalid_data = vec![1.0, f64::NAN, 3.0];

        assert!(processor.validate_data(&valid_data).is_ok());
        assert!(processor.validate_data(&invalid_data).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        let normalized = processor.normalize_data(&data);

        let mean = normalized.iter().sum::<f64>() / normalized.len() as f64;
        assert!(mean.abs() < 1e-10);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];

        let result1 = processor.process_dataset("test", &data);
        let result2 = processor.process_dataset("test", &data);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
        
        let (entries, values) = processor.get_cache_stats();
        assert_eq!(entries, 1);
        assert_eq!(values, 3);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_summary(&self) -> DataSummary {
        let count = self.records.len();
        let mean = self.calculate_mean().unwrap_or(0.0);
        let categories: Vec<String> = self.records
            .iter()
            .map(|r| r.category.clone())
            .collect();
        
        DataSummary {
            record_count: count,
            average_value: mean,
            unique_categories: categories,
        }
    }
}

pub struct DataSummary {
    pub record_count: usize,
    pub average_value: f64,
    pub unique_categories: Vec<String>,
}

impl std::fmt::Display for DataSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Records: {}, Average: {:.2}, Categories: {}",
            self.record_count,
            self.average_value,
            self.unique_categories.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,A").unwrap();
        writeln!(temp_file, "2,20.3,B").unwrap();
        writeln!(temp_file, "3,15.7,A").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let mean = processor.calculate_mean().unwrap();
        assert!((mean - 15.5).abs() < 0.1);
        
        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].trim().to_string();
        let value = parts[2].parse::<f64>()?;
        let active = parts[3].parse::<bool>()?;

        let record = Record::new(id, name, value, active);
        if record.is_valid() {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = sum / count as f64;

    let max_value = records
        .iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);

    (average, max_value, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 20.0, false),
            Record::new(3, "C".to_string(), 30.0, true),
        ];

        let (avg, max, count) = calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert_eq!(max, 30.0);
        assert_eq!(count, 3);
    }
}