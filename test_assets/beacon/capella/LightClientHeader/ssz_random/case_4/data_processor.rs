
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            DataError::MissingMetadata(key) => write!(f, "Missing required metadata: {}", key),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    required_metadata: Vec<String>,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, required_metadata: Vec<String>) -> Self {
        DataProcessor {
            min_value,
            max_value,
            required_metadata,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.min_value || value > self.max_value {
                return Err(DataError::ValueOutOfRange(value));
            }
        }

        for key in &self.required_metadata {
            if !record.metadata.contains_key(key) {
                return Err(DataError::MissingMetadata(key.clone()));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        if let Some(max_val) = record.values.iter().copied().reduce(f64::max) {
            if max_val != 0.0 {
                for value in &mut record.values {
                    *value /= max_val;
                }
            }
        }
    }

    pub fn process_batch(&self, records: &mut [DataRecord]) -> Vec<Result<DataRecord, DataError>> {
        records
            .iter_mut()
            .map(|record| {
                self.validate_record(record)?;
                self.normalize_values(record);
                Ok(record.clone())
            })
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let sum_all: f64 = records.iter().flat_map(|r| &r.values).sum();
        
        stats.insert("mean".to_string(), sum_all / total_values as f64);
        
        let variance: f64 = records
            .iter()
            .flat_map(|r| &r.values)
            .map(|&v| {
                let diff = v - stats["mean"];
                diff * diff
            })
            .sum::<f64>() / total_values as f64;
        
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        
        DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0, 40.0],
            metadata,
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(
            0.0,
            100.0,
            vec!["source".to_string(), "version".to_string()]
        );
        
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_id() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let mut record = create_test_record();
        record.id = 0;
        
        match processor.validate_record(&record) {
            Err(DataError::InvalidId) => (),
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let mut record = create_test_record();
        
        processor.normalize_values(&mut record);
        
        let expected = vec![0.25, 0.5, 0.75, 1.0];
        assert_eq!(record.values, expected);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        
        let mut records = vec![
            create_test_record(),
            {
                let mut r = create_test_record();
                r.id = 2;
                r.values = vec![150.0];
                r
            }
        ];
        
        let results = processor.process_batch(&mut records);
        
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
    }
}