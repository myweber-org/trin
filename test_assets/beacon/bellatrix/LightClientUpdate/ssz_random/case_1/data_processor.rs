use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    MetadataKeyTooLong,
}

pub struct DataProcessor {
    max_metadata_key_length: usize,
}

impl DataProcessor {
    pub fn new(max_metadata_key_length: usize) -> Self {
        Self {
            max_metadata_key_length,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }

        if record.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        for key in record.metadata.keys() {
            if key.len() > self.max_metadata_key_length {
                return Err(ValidationError::MetadataKeyTooLong);
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        if let Some(max_value) = record.values.iter().copied().reduce(f64::max) {
            if max_value != 0.0 {
                for value in record.values.iter_mut() {
                    *value /= max_value;
                }
            }
        }
    }

    pub fn filter_records(
        &self,
        records: Vec<DataRecord>,
        predicate: impl Fn(&DataRecord) -> bool,
    ) -> Vec<DataRecord> {
        records.into_iter().filter(predicate).collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.iter().copied())
            .collect();

        if !total_values.is_empty() {
            let sum: f64 = total_values.iter().sum();
            let count = total_values.len() as f64;
            let mean = sum / count;

            let variance: f64 = total_values
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>()
                / count;

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("total_count".to_string(), count);
            stats.insert("sum".to_string(), sum);
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(50);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_id() {
        let processor = DataProcessor::new(50);
        let record = DataRecord {
            id: 0,
            timestamp: 1000,
            values: vec![1.0],
            metadata: HashMap::new(),
        };

        assert_eq!(
            processor.validate_record(&record),
            Err(ValidationError::InvalidId)
        );
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(50);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![2.0, 4.0, 6.0],
            metadata: HashMap::new(),
        };

        processor.normalize_values(&mut record);
        assert_eq!(record.values, vec![1.0 / 3.0, 2.0 / 3.0, 1.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(50);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![3.0, 4.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = processor.calculate_statistics(&records);
        assert_eq!(stats.get("mean"), Some(&2.5));
        assert_eq!(stats.get("total_count"), Some(&4.0));
    }
}