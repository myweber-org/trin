
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Processing timeout")]
    Timeout,
    #[error("Transformation failed")]
    TransformationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedData {
    pub record_id: u64,
    pub normalized_value: f64,
    pub processed_at: i64,
    pub quality_score: u8,
}

pub struct DataProcessor {
    max_value: f64,
    min_value: f64,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64) -> Result<Self, DataError> {
        if min_value >= max_value {
            return Err(DataError::InvalidInput(
                "Minimum value must be less than maximum value".to_string(),
            ));
        }
        Ok(Self { max_value, min_value })
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.value < self.min_value || record.value > self.max_value {
            return Err(DataError::InvalidInput(format!(
                "Value {} is outside valid range [{}, {}]",
                record.value, self.min_value, self.max_value
            )));
        }
        if record.timestamp < 0 {
            return Err(DataError::InvalidInput("Timestamp cannot be negative".to_string()));
        }
        Ok(())
    }

    pub fn process_record(&self, record: DataRecord) -> Result<ProcessedData, DataError> {
        self.validate_record(&record)?;

        let normalized_value = (record.value - self.min_value) / (self.max_value - self.min_value);
        let quality_score = self.calculate_quality_score(normalized_value);

        Ok(ProcessedData {
            record_id: record.id,
            normalized_value,
            processed_at: chrono::Utc::now().timestamp(),
            quality_score,
        })
    }

    fn calculate_quality_score(&self, normalized_value: f64) -> u8 {
        let score = (normalized_value * 100.0).round() as u8;
        score.clamp(0, 100)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<ProcessedData>, DataError> {
        let mut results = Vec::with_capacity(records.len());
        for record in records {
            match self.process_record(record) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = DataProcessor::new(0.0, 100.0);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(100.0, 0.0);
        assert!(processor.is_err());
    }

    #[test]
    fn test_record_validation() {
        let processor = DataProcessor::new(0.0, 100.0).unwrap();
        let valid_record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
        };
        assert!(processor.validate_record(&valid_record).is_ok());
    }

    #[test]
    fn test_invalid_record_validation() {
        let processor = DataProcessor::new(0.0, 100.0).unwrap();
        let invalid_record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
        };
        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_record_processing() {
        let processor = DataProcessor::new(0.0, 100.0).unwrap();
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1234567890,
        };
        let result = processor.process_record(record);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.record_id, 1);
        assert_eq!(processed.normalized_value, 0.5);
        assert!(processed.quality_score <= 100);
    }
}