
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

    pub fn calculate_statistics(&self) -> Option<Statistics> {
        if self.records.is_empty() {
            return None;
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some(Statistics {
            count,
            mean,
            std_dev,
            min,
            max,
            sum,
        })
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.processed_ids.clear();
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Statistics: count={}, mean={:.2}, std_dev={:.2}, min={:.2}, max={:.2}, sum={:.2}",
            self.count, self.mean, self.std_dev, self.min, self.max, self.sum
        )
    }
}