
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
            DataError::InvalidFormat => write!(f, "Data format is invalid"),
            DataError::OutOfRange => write!(f, "Value is out of acceptable range"),
            DataError::ConversionFailed => write!(f, "Failed to convert data type"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, DataError> {
        if threshold < 0.0 || threshold > 100.0 {
            return Err(DataError::OutOfRange);
        }
        Ok(Self { threshold })
    }

    pub fn process_value(&self, raw_value: &str) -> Result<f64, DataError> {
        let parsed = raw_value.parse::<f64>().map_err(|_| DataError::InvalidFormat)?;
        
        if parsed < 0.0 {
            return Err(DataError::OutOfRange);
        }

        let normalized = (parsed * 100.0) / self.threshold;
        Ok(normalized)
    }

    pub fn batch_process(&self, values: &[&str]) -> Vec<Result<f64, DataError>> {
        values.iter()
            .map(|&v| self.process_value(v))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processing() {
        let processor = DataProcessor::new(50.0).unwrap();
        let result = processor.process_value("25.0").unwrap();
        assert!((result - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_invalid_format() {
        let processor = DataProcessor::new(50.0).unwrap();
        assert!(matches!(processor.process_value("abc"), Err(DataError::InvalidFormat)));
    }

    #[test]
    fn test_out_of_range() {
        let processor = DataProcessor::new(50.0).unwrap();
        assert!(matches!(processor.process_value("-10.0"), Err(DataError::OutOfRange)));
    }
}