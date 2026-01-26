
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
    InvalidValue(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::InvalidValue(val) => write!(f, "Invalid value detected: {}", val),
            DataError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    validation_threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor {
            records: Vec::new(),
            validation_threshold: threshold,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        self.records.push(record);
        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::InvalidValue(value));
            }
        }

        Ok(())
    }

    pub fn process_records(&mut self) -> HashMap<u32, f64> {
        let mut results = HashMap::new();

        for record in &self.records {
            if let Some(avg) = self.calculate_average(&record.values) {
                if avg > self.validation_threshold {
                    results.insert(record.id, avg);
                }
            }
        }

        results
    }

    fn calculate_average(&self, values: &[f64]) -> Option<f64> {
        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        Some(sum / values.len() as f64)
    }

    pub fn filter_by_metadata(&self, key: &str, value: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| {
                record
                    .metadata
                    .get(key)
                    .map_or(false, |v| v == value)
            })
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let mut processor = DataProcessor::new(10.0);
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), "test".to_string());

        let record = DataRecord {
            id: 1,
            values: vec![5.0, 10.0, 15.0],
            metadata,
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_record_count(), 1);
    }

    #[test]
    fn test_invalid_record_rejection() {
        let mut processor = DataProcessor::new(10.0);
        let record = DataRecord {
            id: 0,
            values: vec![5.0, 10.0],
            metadata: HashMap::new(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_average_calculation() {
        let processor = DataProcessor::new(10.0);
        let values = vec![2.0, 4.0, 6.0];
        let avg = processor.calculate_average(&values).unwrap();
        assert_eq!(avg, 4.0);
    }
}