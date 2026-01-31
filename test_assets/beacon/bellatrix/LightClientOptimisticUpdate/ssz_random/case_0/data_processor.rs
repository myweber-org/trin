
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

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_tag(&mut self, tag: &str) {
        self.tags.push(tag.to_string());
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".to_string()));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed("Timestamp cannot be negative".to_string()));
        }

        if self.values.is_empty() {
            return Err(DataError::ValidationFailed("Record must contain at least one value".to_string()));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed("Value key cannot be empty".to_string()));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(format!("Value for '{}' must be finite", key)));
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
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    processed_count: u64,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            processed_count: 0,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        self.processed_count += 1;
        Ok(())
    }

    pub fn process_all(&mut self) {
        for record in &mut self.records {
            record.normalize_values();
        }
    }

    pub fn get_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }

        let total_records = self.records.len() as f64;
        let mut total_values = 0.0;

        for record in &self.records {
            total_values += record.values.values().sum::<f64>();
        }

        stats.insert("total_records".to_string(), total_records);
        stats.insert("total_values".to_string(), total_values);
        stats.insert("avg_values_per_record".to_string(), total_values / total_records);
        stats.insert("processed_count".to_string(), self.processed_count as f64);

        stats
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}