use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Processing timeout")]
    Timeout,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub value: f64,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64, value: f64, tags: Vec<String>) -> Self {
        Self {
            id,
            timestamp,
            value,
            tags,
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.timestamp < 0 {
            return Err(DataError::InvalidInput(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.value.is_nan() || self.value.is_infinite() {
            return Err(DataError::InvalidInput(
                "Value must be a finite number".to_string(),
            ));
        }

        if self.tags.iter().any(|tag| tag.is_empty()) {
            return Err(DataError::InvalidInput(
                "Tags cannot be empty strings".to_string(),
            ));
        }

        Ok(())
    }
}

pub struct DataProcessor {
    max_records: usize,
    processing_timeout: std::time::Duration,
}

impl DataProcessor {
    pub fn new(max_records: usize, timeout_secs: u64) -> Self {
        Self {
            max_records,
            processing_timeout: std::time::Duration::from_secs(timeout_secs),
        }
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, DataError> {
        if records.len() > self.max_records {
            return Err(DataError::InvalidInput(format!(
                "Batch size {} exceeds maximum {}",
                records.len(),
                self.max_records
            )));
        }

        let start_time = std::time::Instant::now();
        let mut processed = Vec::with_capacity(records.len());

        for record in records {
            if start_time.elapsed() > self.processing_timeout {
                return Err(DataError::Timeout);
            }

            record.validate()?;
            let transformed = self.transform_record(record)?;
            processed.push(transformed);
        }

        Ok(processed)
    }

    fn transform_record(&self, mut record: DataRecord) -> Result<DataRecord, DataError> {
        record.value = (record.value * 100.0).round() / 100.0;
        record.tags.sort();
        record.tags.dedup();
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890, 42.5, vec!["tag1".to_string()]);
        assert!(valid_record.validate().is_ok());

        let invalid_timestamp = DataRecord::new(2, -1, 42.5, vec!["tag1".to_string()]);
        assert!(invalid_timestamp.validate().is_err());

        let invalid_value = DataRecord::new(3, 1234567890, f64::NAN, vec!["tag1".to_string()]);
        assert!(invalid_value.validate().is_err());
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(100, 5);
        let records = vec![
            DataRecord::new(1, 1234567890, 42.567, vec!["a".to_string(), "b".to_string()]),
            DataRecord::new(2, 1234567891, 99.999, vec!["b".to_string(), "a".to_string()]),
        ];

        let result = processor.process_batch(records);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].value, 42.57);
        assert_eq!(processed[1].value, 100.0);
        assert_eq!(processed[0].tags, vec!["a", "b"]);
    }
}
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }

        if self.values.is_empty() {
            return Err("Record contains no values".into());
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".into());
            }
        }

        Ok(())
    }

    pub fn transform(&mut self, factor: f64) {
        for value in &mut self.values {
            *value *= factor;
        }
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();

    for record in records {
        record.validate()?;
        let mut transformed = record.clone();
        transformed.transform(factor);
        processed.push(transformed);
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        record.transform(2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}