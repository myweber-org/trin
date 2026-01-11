
use std::error::Error;
use std::fmt;

#[derive(Debug)]
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
        if threshold <= 0.0 {
            return Err(ValidationError {
                message: "Threshold must be positive".to_string(),
            });
        }
        Ok(DataProcessor { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Vec<f64> {
        values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .map(|&v| v * 2.0)
            .collect()
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64) {
        let count = values.len() as f64;
        if count == 0.0 {
            return (0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>()
            / count;

        (mean, variance.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(5.0);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(0.0);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(3.0).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = processor.process_values(&values);
        assert_eq!(result, vec![6.0, 8.0, 10.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(1.0).unwrap();
        let values = vec![2.0, 4.0, 6.0, 8.0];
        let (mean, std_dev) = processor.calculate_statistics(&values);
        assert!((mean - 5.0).abs() < 0.0001);
        assert!((std_dev - 2.236067977).abs() < 0.0001);
    }
}