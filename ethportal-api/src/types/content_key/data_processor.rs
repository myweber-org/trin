
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn display(&self) -> String {
        format!("ID: {}, Value: {:.2}, Category: {}", self.id, self.value, self.category)
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let value = parts[1].parse::<f64>()?;
        let category = parts[2].to_string();

        let record = DataRecord::new(id, value, category);
        if record.is_valid() {
            records.push(record);
        } else {
            eprintln!("Warning: Invalid record at line {}", line_num + 1);
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

    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, "alpha".to_string());
        assert!(record.is_valid());
        assert_eq!(record.display(), "ID: 1, Value: 42.50, Category: alpha");
    }

    #[test]
    fn test_invalid_record() {
        let record1 = DataRecord::new(0, 10.0, "test".to_string());
        let record2 = DataRecord::new(1, -5.0, "test".to_string());
        let record3 = DataRecord::new(1, 10.0, "".to_string());
        
        assert!(!record1.is_valid());
        assert!(!record2.is_valid());
        assert!(!record3.is_valid());
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, "a".to_string()),
            DataRecord::new(2, 20.0, "b".to_string()),
            DataRecord::new(3, 30.0, "c".to_string()),
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn add_value(&mut self, key: &str, value: f64) -> Result<(), DataError> {
        if key.trim().is_empty() {
            return Err(DataError::InvalidInput("Key cannot be empty".to_string()));
        }
        if !value.is_finite() {
            return Err(DataError::InvalidInput("Value must be finite".to_string()));
        }
        self.values.insert(key.to_string(), value);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: &str) -> Result<(), DataError> {
        if tag.trim().is_empty() {
            return Err(DataError::InvalidInput("Tag cannot be empty".to_string()));
        }
        if self.tags.len() >= 10 {
            return Err(DataError::ValidationError("Maximum 10 tags allowed".to_string()));
        }
        self.tags.push(tag.to_string());
        Ok(())
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.values.is_empty() {
            return Err(DataError::ValidationError("Record must contain at least one value".to_string()));
        }
        if self.timestamp < 0 {
            return Err(DataError::ValidationError("Timestamp cannot be negative".to_string()));
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if !self.values.is_empty() {
            let count = self.values.len() as f64;
            let sum: f64 = self.values.values().sum();
            let avg = sum / count;
            
            let variance: f64 = self.values.values()
                .map(|v| (v - avg).powi(2))
                .sum::<f64>() / count;
            
            stats.insert("count".to_string(), count);
            stats.insert("sum".to_string(), sum);
            stats.insert("average".to_string(), avg);
            stats.insert("variance".to_string(), variance);
        }
        
        stats
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

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn process_records(&self) -> Result<HashMap<String, f64>, DataError> {
        if self.records.is_empty() {
            return Err(DataError::ProcessingFailed("No records to process".to_string()));
        }

        let mut aggregated = HashMap::new();
        let mut total_records = 0;

        for record in &self.records {
            let stats = record.calculate_statistics();
            for (key, value) in stats {
                *aggregated.entry(key).or_insert(0.0) += value;
            }
            total_records += 1;
        }

        if total_records > 0 {
            for value in aggregated.values_mut() {
                *value /= total_records as f64;
            }
        }

        Ok(aggregated)
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 1234567890);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert!(record.values.is_empty());
        assert!(record.tags.is_empty());
    }

    #[test]
    fn test_add_value() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.add_value("temperature", 25.5).is_ok());
        assert_eq!(record.values.get("temperature"), Some(&25.5));
    }

    #[test]
    fn test_add_tag() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.add_tag("sensor").is_ok());
        assert!(record.tags.contains(&"sensor".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.validate().is_err());
        
        record.add_value("test", 1.0).unwrap();
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("value", 10.0).unwrap();
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_record_count(), 1);
    }
}