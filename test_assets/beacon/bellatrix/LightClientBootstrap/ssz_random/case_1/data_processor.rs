
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ProcessingError {
    message: String,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Processing error: {}", self.message)
    }
}

impl Error for ProcessingError {}

impl ProcessingError {
    pub fn new(msg: &str) -> Self {
        ProcessingError {
            message: msg.to_string(),
        }
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, ProcessingError> {
        if id == 0 {
            return Err(ProcessingError::new("ID cannot be zero"));
        }
        if !value.is_finite() {
            return Err(ProcessingError::new("Value must be finite"));
        }
        if timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }

        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(ProcessingError::new("Value out of valid range (0-1000)"));
        }
        Ok(())
    }
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<f64>, ProcessingError> {
    if records.is_empty() {
        return Err(ProcessingError::new("No records to process"));
    }

    let mut results = Vec::with_capacity(records.len());
    for record in records {
        record.validate()?;
        let processed_value = transform_value(record.value)?;
        results.push(processed_value);
    }

    Ok(results)
}

fn transform_value(value: f64) -> Result<f64, ProcessingError> {
    if value <= 0.0 {
        return Err(ProcessingError::new("Value must be positive for transformation"));
    }

    let transformed = (value * 2.5).ln() / (value + 1.0).sqrt();
    if transformed.is_nan() || transformed.is_infinite() {
        return Err(ProcessingError::new("Transformation produced invalid result"));
    }

    Ok(transformed)
}

pub fn calculate_statistics(values: &[f64]) -> Result<(f64, f64, f64), ProcessingError> {
    if values.is_empty() {
        return Err(ProcessingError::new("Cannot calculate statistics for empty dataset"));
    }

    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;

    let variance: f64 = values
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;

    let std_dev = variance.sqrt();

    Ok((mean, variance, std_dev))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 50.5, 1234567890).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 50.5);
        assert_eq!(record.timestamp, 1234567890);
    }

    #[test]
    fn test_invalid_record_creation() {
        assert!(DataRecord::new(0, 50.5, 1234567890).is_err());
        assert!(DataRecord::new(1, f64::INFINITY, 1234567890).is_err());
        assert!(DataRecord::new(1, 50.5, -1).is_err());
    }

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 500.0, 1234567890).unwrap();
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(1, 1500.0, 1234567890).unwrap();
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 100.0, 1234567890).unwrap(),
            DataRecord::new(2, 200.0, 1234567891).unwrap(),
        ];

        let results = process_records(&records).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].is_finite());
        assert!(results[1].is_finite());
    }

    #[test]
    fn test_calculate_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = calculate_statistics(&values).unwrap();

        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: u64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid value field"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: u64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if !value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        if timestamp == 0 {
            return Err(DataError::InvalidTimestamp);
        }

        Ok(Self {
            id,
            value,
            timestamp,
        })
    }

    pub fn transform(&self, factor: f64) -> Result<Self, DataError> {
        if !factor.is_finite() || factor <= 0.0 {
            return Err(DataError::TransformationError(
                "Invalid transformation factor".to_string(),
            ));
        }

        let transformed_value = self.value * factor;
        Ok(Self {
            id: self.id,
            value: transformed_value,
            timestamp: self.timestamp,
        })
    }

    pub fn normalize(&self, max_value: f64) -> Result<f64, DataError> {
        if max_value <= 0.0 || !max_value.is_finite() {
            return Err(DataError::TransformationError(
                "Invalid max value for normalization".to_string(),
            ));
        }

        if self.value > max_value {
            return Err(DataError::TransformationError(
                "Value exceeds maximum allowed".to_string(),
            ));
        }

        Ok(self.value / max_value)
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<DataRecord, DataError>> {
    records
        .iter()
        .map(|record| record.transform(2.0))
        .collect()
}

pub fn validate_record_batch(records: &[DataRecord]) -> Result<(), DataError> {
    for record in records {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        if !record.value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        if record.timestamp == 0 {
            return Err(DataError::InvalidTimestamp);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1234567890);
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 42.5, 1234567890);
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord::new(1, 10.0, 1234567890).unwrap();
        let transformed = record.transform(2.0);
        assert!(transformed.is_ok());
        assert_eq!(transformed.unwrap().value, 20.0);
    }

    #[test]
    fn test_normalize_record() {
        let record = DataRecord::new(1, 50.0, 1234567890).unwrap();
        let normalized = record.normalize(100.0);
        assert!(normalized.is_ok());
        assert_eq!(normalized.unwrap(), 0.5);
    }
}