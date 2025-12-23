
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid numeric value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::InvalidValue);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        if record.value > self.threshold {
            return Err(ProcessingError::ValidationFailed(
                format!("Value {} exceeds threshold {}", record.value, self.threshold)
            ));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> DataRecord {
        DataRecord {
            id: record.id,
            value: record.value * 2.0,
            timestamp: record.timestamp + 3600,
        }
    }

    pub fn process_records(&self, records: &[DataRecord]) -> Vec<Result<DataRecord, ProcessingError>> {
        records.iter()
            .map(|record| {
                self.validate_record(record)?;
                Ok(self.transform_record(record))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1609459200,
        };

        let result = processor.process_records(&[record.clone()]);
        assert!(result[0].is_ok());
        
        let processed = result[0].as_ref().unwrap();
        assert_eq!(processed.value, 100.0);
        assert_eq!(processed.timestamp, 1609459200 + 3600);
    }

    #[test]
    fn test_invalid_value() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1609459200,
        };

        let result = processor.process_records(&[record]);
        assert!(result[0].is_err());
    }

    #[test]
    fn test_threshold_exceeded() {
        let processor = DataProcessor::new(50.0);
        let record = DataRecord {
            id: 1,
            value: 75.0,
            timestamp: 1609459200,
        };

        let result = processor.process_records(&[record]);
        assert!(result[0].is_err());
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }

        Ok(DataProcessor { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: "Input values cannot be empty".to_string(),
            });
        }

        let filtered: Vec<f64> = values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .cloned()
            .collect();

        if filtered.is_empty() {
            return Err(ValidationError {
                message: format!(
                    "No values meet the threshold requirement of {}",
                    self.threshold
                ),
            });
        }

        let mean = filtered.iter().sum::<f64>() / filtered.len() as f64;
        let normalized: Vec<f64> = filtered.iter().map(|&v| v / mean).collect();

        Ok(normalized)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64, f64), ValidationError> {
        if data.len() < 2 {
            return Err(ValidationError {
                message: "Insufficient data for statistics calculation".to_string(),
            });
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data
            .iter()
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / (data.len() - 1) as f64;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());

        let invalid = DataProcessor::new(1.5);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(0.3).unwrap();
        let values = vec![0.1, 0.4, 0.5, 0.2, 0.6];
        let result = processor.process_values(&values);

        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.0).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&data);

        assert!(stats.is_ok());
        let (mean, variance, std_dev) = stats.unwrap();
        assert!((mean - 3.0).abs() < 1e-10);
        assert!((variance - 2.5).abs() < 1e-10);
        assert!((std_dev - 1.58113883).abs() < 1e-6);
    }
}