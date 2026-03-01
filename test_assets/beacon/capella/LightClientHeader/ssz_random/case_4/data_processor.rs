
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
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            DataError::MissingMetadata(key) => write!(f, "Missing required metadata: {}", key),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    required_metadata: Vec<String>,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, required_metadata: Vec<String>) -> Self {
        DataProcessor {
            min_value,
            max_value,
            required_metadata,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.min_value || value > self.max_value {
                return Err(DataError::ValueOutOfRange(value));
            }
        }

        for key in &self.required_metadata {
            if !record.metadata.contains_key(key) {
                return Err(DataError::MissingMetadata(key.clone()));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        if let Some(max_val) = record.values.iter().copied().reduce(f64::max) {
            if max_val != 0.0 {
                for value in &mut record.values {
                    *value /= max_val;
                }
            }
        }
    }

    pub fn process_batch(&self, records: &mut [DataRecord]) -> Vec<Result<DataRecord, DataError>> {
        records
            .iter_mut()
            .map(|record| {
                self.validate_record(record)?;
                self.normalize_values(record);
                Ok(record.clone())
            })
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let sum_all: f64 = records.iter().flat_map(|r| &r.values).sum();
        
        stats.insert("mean".to_string(), sum_all / total_values as f64);
        
        let variance: f64 = records
            .iter()
            .flat_map(|r| &r.values)
            .map(|&v| {
                let diff = v - stats["mean"];
                diff * diff
            })
            .sum::<f64>() / total_values as f64;
        
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        
        DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0, 40.0],
            metadata,
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(
            0.0,
            100.0,
            vec!["source".to_string(), "version".to_string()]
        );
        
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_id() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let mut record = create_test_record();
        record.id = 0;
        
        match processor.validate_record(&record) {
            Err(DataError::InvalidId) => (),
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let mut record = create_test_record();
        
        processor.normalize_values(&mut record);
        
        let expected = vec![0.25, 0.5, 0.75, 1.0];
        assert_eq!(record.values, expected);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        
        let mut records = vec![
            create_test_record(),
            {
                let mut r = create_test_record();
                r.id = 2;
                r.values = vec![150.0];
                r
            }
        ];
        
        let results = processor.process_batch(&mut records);
        
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut loaded_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].trim().to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[3].trim().to_string();

            let record = DataRecord::new(id, name, value, category);
            if record.is_valid() {
                self.records.push(record);
                loaded_count += 1;
            }
        }

        Ok(loaded_count)
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

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_records().len(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.0,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.75,CategoryA").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.get_records().len(), 3);

        let category_a = processor.filter_by_category("CategoryA");
        assert_eq!(category_a.len(), 2);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 15.416).abs() < 0.001);

        processor.clear();
        assert_eq!(processor.get_records().len(), 0);
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

    pub fn process_numeric_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data array provided".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }

        let processed: Vec<f64> = values
            .iter()
            .map(|&x| x * 2.0)
            .filter(|&x| x > 0.0)
            .collect();

        if processed.is_empty() {
            return Err("All values filtered out during processing".to_string());
        }

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            (mean, variance, std_dev)
        })
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cached_keys(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
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
        assert_eq!(processed, vec![2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_numeric_data("invalid", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        processor.process_numeric_data("stats", &data).unwrap();
        let stats = processor.calculate_statistics("stats");
        
        assert!(stats.is_some());
        let (mean, variance, std_dev) = stats.unwrap();
        
        assert!((mean - 6.0).abs() < 0.001);
        assert!((variance - 8.0).abs() < 0.001);
        assert!((std_dev - 2.828).abs() < 0.001);
    }
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Transformation failed")]
    TransformationFailed,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: String, value: f64) -> Result<(), DataError> {
        if value.is_nan() || value.is_infinite() {
            return Err(DataError::InvalidInput);
        }
        self.values.insert(key, value);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.values.is_empty() {
            return Err(DataError::ValidationError(
                "Record must contain at least one value".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        if multiplier <= 0.0 || multiplier.is_nan() || multiplier.is_infinite() {
            return Err(DataError::TransformationFailed);
        }

        for value in self.values.values_mut() {
            *value *= multiplier;
        }

        Ok(())
    }
}

pub fn process_records(
    records: Vec<DataRecord>,
    multiplier: f64,
) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for mut record in records {
        record.validate()?;
        record.transform(multiplier)?;
        processed.push(record);
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 1234567890);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert!(record.values.is_empty());
        assert!(record.tags.is_empty());
    }

    #[test]
    fn test_add_valid_value() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.add_value("temperature".to_string(), 25.5).is_ok());
        assert_eq!(record.values.get("temperature"), Some(&25.5));
    }

    #[test]
    fn test_add_invalid_value() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.add_value("invalid".to_string(), f64::NAN).is_err());
    }

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.validate().is_err());

        record.add_value("test".to_string(), 1.0).unwrap();
        assert!(record.validate().is_ok());

        let mut invalid_record = DataRecord::new(2, -1);
        invalid_record.add_value("test".to_string(), 1.0).unwrap();
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("value1".to_string(), 10.0).unwrap();
        record.add_value("value2".to_string(), 20.0).unwrap();

        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.values.get("value1"), Some(&20.0));
        assert_eq!(record.values.get("value2"), Some(&40.0));
    }

    #[test]
    fn test_process_records() {
        let mut record1 = DataRecord::new(1, 1234567890);
        record1.add_value("temp".to_string(), 10.0).unwrap();

        let mut record2 = DataRecord::new(2, 1234567891);
        record2.add_value("temp".to_string(), 20.0).unwrap();

        let records = vec![record1, record2];
        let processed = process_records(records, 3.0).unwrap();

        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values.get("temp"), Some(&30.0));
        assert_eq!(processed[1].values.get("temp"), Some(&60.0));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
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

    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
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

            let record = DataRecord::new(id, value, parts[2]);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    pub fn get_statistics(&self) -> Statistics {
        let valid_records = self.filter_valid();
        let count = valid_records.len();
        
        if count == 0 {
            return Statistics::empty();
        }

        let values: Vec<f64> = valid_records.iter().map(|r| r.value).collect();
        let sum: f64 = values.iter().sum();
        let avg = sum / count as f64;
        
        let variance: f64 = values.iter()
            .map(|v| (v - avg).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Statistics {
            count,
            sum,
            average: avg,
            min,
            max,
            standard_deviation: std_dev,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub standard_deviation: f64,
}

impl Statistics {
    pub fn empty() -> Self {
        Statistics {
            count: 0,
            sum: 0.0,
            average: 0.0,
            min: 0.0,
            max: 0.0,
            standard_deviation: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "A");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "A");
        assert!(record.valid);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -10.0, "B");
        assert!(!record.valid);
    }

    #[test]
    fn test_csv_loading() {
        let mut csv_content = "id,value,category\n".to_string();
        csv_content.push_str("1,42.5,A\n");
        csv_content.push_str("2,15.3,B\n");
        csv_content.push_str("3,-5.0,C\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.records.len(), 3);
    }

    #[test]
    fn test_filter_valid() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A"));
        processor.records.push(DataRecord::new(2, -5.0, "B"));
        processor.records.push(DataRecord::new(3, 20.0, "C"));
        
        let valid = processor.filter_valid();
        assert_eq!(valid.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A"));
        processor.records.push(DataRecord::new(2, 20.0, "B"));
        processor.records.push(DataRecord::new(3, 30.0, "C"));
        
        let avg = processor.calculate_average();
        assert_eq!(avg, Some(20.0));
    }

    #[test]
    fn test_group_by_category() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A"));
        processor.records.push(DataRecord::new(2, 20.0, "A"));
        processor.records.push(DataRecord::new(3, 30.0, "B"));
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("A").unwrap().len(), 2);
        assert_eq!(groups.get("B").unwrap().len(), 1);
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A"));
        processor.records.push(DataRecord::new(2, 20.0, "B"));
        processor.records.push(DataRecord::new(3, 30.0, "C"));
        
        let stats = processor.get_statistics();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 60.0);
        assert_eq!(stats.average, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
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

pub fn process_csv_file(file_path: &Path) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid format at line {}", line_number + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].to_string();
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

    let variance: f64 = records
        .iter()
        .map(|r| (r.value - average).powi(2))
        .sum::<f64>()
        / count as f64;

    (average, variance, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_process_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,Alice,100.5,true")?;
        writeln!(temp_file, "2,Bob,75.2,false")?;
        writeln!(temp_file, "# This is a comment")?;
        writeln!(temp_file, "")?;

        let records = process_csv_file(temp_file.path())?;
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[1].name, "Bob");

        Ok(())
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 20.0, false),
            Record::new(3, "C".to_string(), 30.0, true),
        ];

        let (avg, var, count) = calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert_eq!(var, 66.66666666666667);
        assert_eq!(count, 3);
    }
}