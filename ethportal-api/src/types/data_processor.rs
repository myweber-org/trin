
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data value: {0}")]
    InvalidValue(String),
    #[error("Timestamp out of range")]
    InvalidTimestamp,
    #[error("Serialization error")]
    SerializationFailed,
}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        DataProcessor { min_value, max_value }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(ProcessingError::InvalidValue(
                format!("Value {} outside range [{}, {}]", record.value, self.min_value, self.max_value)
            ));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        Ok(())
    }

    pub fn normalize_value(&self, record: &DataRecord) -> f64 {
        (record.value - self.min_value) / (self.max_value - self.min_value)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<f64>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let normalized = self.normalize_value(&record);
            results.push(normalized);
        }
        
        Ok(results)
    }

    pub fn serialize_records(records: &[DataRecord]) -> Result<String, ProcessingError> {
        serde_json::to_string(records)
            .map_err(|_| ProcessingError::SerializationFailed)
    }

    pub fn deserialize_records(data: &str) -> Result<Vec<DataRecord>, ProcessingError> {
        serde_json::from_str(data)
            .map_err(|_| ProcessingError::SerializationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new(0.0, 100.0);
        let valid_record = DataRecord { id: 1, value: 50.0, timestamp: 1234567890 };
        let invalid_record = DataRecord { id: 2, value: 150.0, timestamp: 1234567890 };
        
        assert!(processor.validate_record(&valid_record).is_ok());
        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord { id: 1, value: 75.0, timestamp: 1234567890 };
        
        assert_eq!(processor.normalize_value(&record), 0.75);
    }

    #[test]
    fn test_serialization() {
        let records = vec![
            DataRecord { id: 1, value: 25.0, timestamp: 1234567890 },
            DataRecord { id: 2, value: 75.0, timestamp: 1234567891 },
        ];
        
        let serialized = DataProcessor::serialize_records(&records);
        assert!(serialized.is_ok());
        
        let deserialized = DataProcessor::deserialize_records(&serialized.unwrap());
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap().len(), 2);
    }
}