
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
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

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidFormat);
        }

        if self.timestamp < 0 {
            return Err(DataError::OutOfRange("timestamp".to_string()));
        }

        if self.values.is_empty() {
            return Err(DataError::MissingField("values".to_string()));
        }

        Ok(())
    }

    pub fn add_value(&mut self, key: String, value: f64) {
        self.values.insert(key, value);
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn get_value(&self, key: &str) -> Option<f64> {
        self.values.get(key).copied()
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::new();

    for record in records {
        record.validate()?;
        
        let mut processed_record = record.clone();
        
        for (key, value) in &processed_record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::InvalidFormat);
            }
        }

        processed_record.add_tag("processed".to_string());
        processed.push(processed_record);
    }

    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();

    if records.is_empty() {
        return stats;
    }

    let mut value_sums: HashMap<String, f64> = HashMap::new();
    let mut value_counts: HashMap<String, usize> = HashMap::new();

    for record in records {
        for (key, value) in &record.values {
            *value_sums.entry(key.clone()).or_insert(0.0) += value;
            *value_counts.entry(key.clone()).or_insert(0) += 1;
        }
    }

    for (key, sum) in value_sums {
        if let Some(&count) = value_counts.get(&key) {
            if count > 0 {
                stats.insert(format!("{}_average", key), sum / count as f64);
            }
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature".to_string(), 25.5);
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, 1234567890);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("pressure".to_string(), 1013.25);
        
        let result = process_records(vec![record]);
        assert!(result.is_ok());
        
        if let Ok(records) = result {
            assert_eq!(records.len(), 1);
            assert!(records[0].has_tag("processed"));
        }
    }

    #[test]
    fn test_calculate_statistics() {
        let mut record1 = DataRecord::new(1, 1234567890);
        record1.add_value("temperature".to_string(), 20.0);
        
        let mut record2 = DataRecord::new(2, 1234567891);
        record2.add_value("temperature".to_string(), 30.0);
        
        let stats = calculate_statistics(&[record1, record2]);
        assert_eq!(stats.get("temperature_average"), Some(&25.0));
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
    MissingField,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::MissingField => write!(f, "Required field is missing"),
            DataError::DuplicateRecord => write!(f, "Duplicate record detected"),
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
        
        if record.value < 0.0 {
            return Err(DataError::InvalidValue);
        }
        
        if record.name.is_empty() || record.category.is_empty() {
            return Err(DataError::MissingField);
        }
        
        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }
        
        self.records.insert(record.id, record.clone());
        
        let total = self.category_totals.entry(record.category.clone())
            .or_insert(0.0);
        *total += record.value;
        
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_total(&self, category: &str) -> f64 {
        self.category_totals.get(category).copied().unwrap_or(0.0)
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.category_totals.clear();
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
            category: "A".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.total_records(), 1);
    }

    #[test]
    fn test_add_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 50.0,
            category: "B".to_string(),
        };
        
        let record2 = DataRecord {
            id: 1,
            name: "Second".to_string(),
            value: 75.0,
            category: "C".to_string(),
        };
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
    }

    #[test]
    fn test_category_totals() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 10.0, category: "X".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 20.0, category: "X".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 30.0, category: "Y".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.get_category_total("X"), 30.0);
        assert_eq!(processor.get_category_total("Y"), 30.0);
        assert_eq!(processor.get_category_total("Z"), 0.0);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "A".to_string(), value: 10.0, category: "T".to_string() },
            DataRecord { id: 2, name: "B".to_string(), value: 20.0, category: "T".to_string() },
            DataRecord { id: 3, name: "C".to_string(), value: 30.0, category: "T".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.calculate_average_value(), 20.0);
    }
}