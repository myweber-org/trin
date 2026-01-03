
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid data value: {0}")]
    InvalidValue(f64),
    #[error("Timestamp out of range: {0}")]
    InvalidTimestamp(i64),
    #[error("Duplicate record ID: {0}")]
    DuplicateId(u32),
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    processed_ids: std::collections::HashSet<u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            processed_ids: std::collections::HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue(record.value));
        }

        if record.timestamp < 0 || record.timestamp > 253402300799 {
            return Err(DataError::InvalidTimestamp(record.timestamp));
        }

        if self.processed_ids.contains(&record.id) {
            return Err(DataError::DuplicateId(record.id));
        }

        self.processed_ids.insert(record.id);
        self.records.push(record);
        Ok(())
    }

    pub fn process_records(&mut self) -> Vec<DataRecord> {
        self.records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        let mut processed = Vec::new();
        for record in &self.records {
            let transformed = DataRecord {
                id: record.id,
                value: record.value * 1.1,
                timestamp: record.timestamp,
            };
            processed.push(transformed);
        }
        
        processed
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}