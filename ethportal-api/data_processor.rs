
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DataError {
    InvalidFormat,
    OutOfRange,
    ConversionFailed,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidFormat => write!(f, "Invalid data format"),
            DataError::OutOfRange => write!(f, "Value out of acceptable range"),
            DataError::ConversionFailed => write!(f, "Data conversion failed"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, DataError> {
        if threshold <= 0.0 {
            return Err(DataError::OutOfRange);
        }
        Ok(Self { threshold })
    }

    pub fn validate_input(&self, value: f64) -> Result<f64, DataError> {
        if value.is_nan() || value.is_infinite() {
            return Err(DataError::InvalidFormat);
        }
        if value < 0.0 || value > self.threshold {
            return Err(DataError::OutOfRange);
        }
        Ok(value)
    }

    pub fn transform_data(&self, input: f64) -> Result<f64, DataError> {
        let validated = self.validate_input(input)?;
        let transformed = validated.log10() * 100.0;
        
        if transformed.is_nan() || transformed.is_infinite() {
            Err(DataError::ConversionFailed)
        } else {
            Ok(transformed)
        }
    }

    pub fn process_batch(&self, data: &[f64]) -> Vec<Result<f64, DataError>> {
        data.iter()
            .map(|&value| self.transform_data(value))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processing() {
        let processor = DataProcessor::new(1000.0).unwrap();
        let result = processor.transform_data(100.0);
        assert!(result.is_ok());
        assert!((result.unwrap() - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_invalid_input() {
        let processor = DataProcessor::new(100.0).unwrap();
        assert!(processor.transform_data(-10.0).is_err());
        assert!(processor.transform_data(150.0).is_err());
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(500.0).unwrap();
        let data = vec![10.0, 100.0, 500.0, -5.0];
        let results = processor.process_batch(&data);
        
        assert_eq!(results.len(), 4);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_ok());
        assert!(results[3].is_err());
    }
}