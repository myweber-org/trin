
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
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
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError(
                "ID cannot be zero".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(ProcessingError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Values cannot be empty".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(ProcessingError::ValidationError(
                    "Key cannot be empty".to_string(),
                ));
            }
            if !value.is_finite() {
                return Err(ProcessingError::ValidationError(
                    format!("Value for key '{}' must be finite", key),
                ));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&mut self) -> Result<(), ProcessingError> {
        let sum: f64 = self.values.values().sum();
        if sum.abs() < f64::EPSILON {
            return Err(ProcessingError::TransformationFailed(
                "Cannot normalize zero sum".to_string(),
            ));
        }

        for value in self.values.values_mut() {
            *value /= sum;
        }

        Ok(())
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> HashMap<String, f64> {
        self.values
            .iter()
            .filter(|(_, &value)| value >= threshold)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}

pub struct DataProcessor {
    pub records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn process_all(&mut self) -> Result<(), ProcessingError> {
        for record in &mut self.records {
            record.normalize_values()?;
        }
        Ok(())
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }

        let total_records = self.records.len() as f64;
        
        let total_values: usize = self.records.iter().map(|r| r.values.len()).sum();
        stats.insert("average_values_per_record".to_string(), total_values as f64 / total_records);

        let mut total_sum = 0.0;
        let mut count = 0;
        
        for record in &self.records {
            for value in record.values.values() {
                total_sum += value;
                count += 1;
            }
        }

        if count > 0 {
            stats.insert("average_value".to_string(), total_sum / count as f64);
        }

        stats
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
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
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("temp".to_string(), 25.5)]),
            tags: vec!["sensor".to_string()],
        };

        assert!(record.validate().is_ok());

        record.id = 0;
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_normalize_values() {
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

        assert!(record.normalize_values().is_ok());
        
        let sum: f64 = record.values.values().sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_filter_by_threshold() {
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("a".to_string(), 0.1),
                ("b".to_string(), 0.5),
                ("c".to_string(), 0.9),
            ]),
            tags: vec![],
        };

        let filtered = record.filter_by_threshold(0.5);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains_key("b"));
        assert!(filtered.contains_key("c"));
    }
}