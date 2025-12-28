
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ProcessingError {
    InvalidInput(String),
    TransformationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ProcessingError> {
        if threshold <= 0.0 {
            return Err(ProcessingError::InvalidInput(
                "Threshold must be positive".to_string(),
            ));
        }
        Ok(DataProcessor { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        if values.is_empty() {
            return Err(ProcessingError::InvalidInput("Empty input array".to_string()));
        }

        let mut result = Vec::with_capacity(values.len());
        for &value in values {
            if value < 0.0 {
                return Err(ProcessingError::InvalidInput(format!(
                    "Negative value encountered: {}",
                    value
                )));
            }

            let processed = self.apply_transformation(value)?;
            result.push(processed);
        }
        Ok(result)
    }

    fn apply_transformation(&self, value: f64) -> Result<f64, ProcessingError> {
        let transformed = (value * value).sqrt() / self.threshold;
        
        if transformed.is_nan() || transformed.is_infinite() {
            Err(ProcessingError::TransformationFailed(
                "Numerical overflow during transformation".to_string(),
            ))
        } else {
            Ok(transformed)
        }
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64), ProcessingError> {
        let processed = self.process_values(values)?;
        
        let sum: f64 = processed.iter().sum();
        let mean = sum / processed.len() as f64;
        
        let variance: f64 = processed
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / processed.len() as f64;
        
        Ok((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processing() {
        let processor = DataProcessor::new(2.0).unwrap();
        let values = vec![1.0, 4.0, 9.0];
        let result = processor.process_values(&values).unwrap();
        assert_eq!(result, vec![0.5, 2.0, 4.5]);
    }

    #[test]
    fn test_invalid_threshold() {
        let result = DataProcessor::new(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(1.0).unwrap();
        let values = vec![2.0, 4.0, 6.0];
        let (mean, std_dev) = processor.calculate_statistics(&values).unwrap();
        assert!((mean - 4.0).abs() < 1e-10);
        assert!((std_dev - 2.0).abs() < 1e-10);
    }
}