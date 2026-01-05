
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            DataError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>, metadata: HashMap<String, String>) -> Self {
        DataRecord {
            id,
            values,
            metadata,
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidId);
        }

        if self.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &self.values {
            if !value.is_finite() || value < 0.0 || value > 1000.0 {
                return Err(DataError::ValueOutOfRange(value));
            }
        }

        if !self.metadata.contains_key("source") {
            return Err(DataError::MissingMetadata("source".to_string()));
        }

        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) -> &mut Self {
        for value in &mut self.values {
            *value *= multiplier;
        }
        self
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<(u32, f64, f64, f64)>, DataError> {
    let mut results = Vec::new();

    for record in records {
        record.validate()?;
        record.transform(multiplier);
        
        let (mean, variance, std_dev) = record.calculate_statistics();
        results.push((record.id, mean, variance, std_dev));
    }

    Ok(results)
}

pub fn filter_records_by_threshold(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| {
            let (mean, _, _) = record.calculate_statistics();
            mean >= threshold
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        metadata.insert("timestamp".to_string(), "2024-01-01".to_string());

        DataRecord::new(
            1,
            vec![10.0, 20.0, 30.0, 40.0, 50.0],
            metadata,
        )
    }

    #[test]
    fn test_record_validation() {
        let record = create_test_record();
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = create_test_record();
        record.transform(2.0);
        
        let expected_values = vec![20.0, 40.0, 60.0, 80.0, 100.0];
        assert_eq!(record.values, expected_values);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = create_test_record();
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert_eq!(mean, 30.0);
        assert_eq!(variance, 200.0);
        assert_eq!(std_dev, 200.0_f64.sqrt());
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![create_test_record()];
        let result = process_records(&mut records, 1.0);
        
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.len(), 1);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![create_test_record()];
        let filtered = filter_records_by_threshold(&records, 25.0);
        
        assert_eq!(filtered.len(), 1);
    }
}