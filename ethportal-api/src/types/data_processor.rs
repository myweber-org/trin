
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DataError {
    InvalidFormat,
    OutOfRange,
    EmptyData,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidFormat => write!(f, "Data format is invalid"),
            DataError::OutOfRange => write!(f, "Value is out of acceptable range"),
            DataError::EmptyData => write!(f, "No data provided"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, DataError> {
        if threshold <= 0.0 || threshold > 100.0 {
            return Err(DataError::OutOfRange);
        }
        Ok(Self { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, DataError> {
        if values.is_empty() {
            return Err(DataError::EmptyData);
        }

        let mut result = Vec::with_capacity(values.len());
        for &value in values {
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::InvalidFormat);
            }
            let processed = if value > self.threshold {
                value * 0.9
            } else {
                value * 1.1
            };
            result.push(processed);
        }
        Ok(result)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64, f64), DataError> {
        if values.is_empty() {
            return Err(DataError::EmptyData);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(50.0);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_threshold() {
        let processor = DataProcessor::new(0.0);
        assert!(processor.is_err());
        
        let processor = DataProcessor::new(150.0);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(50.0).unwrap();
        let values = vec![30.0, 60.0, 45.0, 70.0];
        let result = processor.process_values(&values);
        
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        
        for (original, processed) in values.iter().zip(processed.iter()) {
            if *original > 50.0 {
                assert!(*processed < *original);
            } else {
                assert!(*processed > *original);
            }
        }
    }

    #[test]
    fn test_empty_data() {
        let processor = DataProcessor::new(50.0).unwrap();
        let result = processor.process_values(&[]);
        assert!(result.is_err());
        
        if let Err(e) = result {
            match e {
                DataError::EmptyData => assert!(true),
                _ => panic!("Wrong error type"),
            }
        }
    }
}