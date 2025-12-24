
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
        if value < 0.0 || value > 1000.0 {
            return Err(ProcessingError::new("Value must be between 0 and 1000"));
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
}

pub fn normalize_value(value: f64) -> f64 {
    (value - 500.0) / 500.0
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<f64>, ProcessingError> {
    if records.is_empty() {
        return Err(ProcessingError::new("No records to process"));
    }

    let mut results = Vec::with_capacity(records.len());
    for record in records {
        let normalized = normalize_value(record.value);
        results.push(normalized);
    }

    Ok(results)
}

pub fn calculate_statistics(values: &[f64]) -> (f64, f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;
    
    let variance: f64 = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, 1625097600);
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.timestamp, 1625097600);
    }

    #[test]
    fn test_invalid_value_record() {
        let record = DataRecord::new(1, -10.0, 1625097600);
        assert!(record.is_err());
    }

    #[test]
    fn test_normalize_value() {
        assert_eq!(normalize_value(500.0), 0.0);
        assert_eq!(normalize_value(1000.0), 1.0);
        assert_eq!(normalize_value(0.0), -1.0);
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 500.0, 1625097600).unwrap(),
            DataRecord::new(2, 750.0, 1625097601).unwrap(),
            DataRecord::new(3, 250.0, 1625097602).unwrap(),
        ];
        
        let result = process_records(records);
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], 0.0);
        assert_eq!(values[1], 0.5);
        assert_eq!(values[2], -0.5);
    }

    #[test]
    fn test_calculate_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = calculate_statistics(&values);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
    }
}