
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error for field '{}': {}", self.field, self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    pub threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_input(&self, value: f64) -> Result<(), ValidationError> {
        if value.is_nan() {
            return Err(ValidationError {
                field: "input".to_string(),
                message: "Value cannot be NaN".to_string(),
            });
        }

        if value.is_infinite() {
            return Err(ValidationError {
                field: "input".to_string(),
                message: "Value cannot be infinite".to_string(),
            });
        }

        if value < 0.0 {
            return Err(ValidationError {
                field: "input".to_string(),
                message: "Value must be non-negative".to_string(),
            });
        }

        Ok(())
    }

    pub fn process_data(&self, data: &[f64]) -> Result<Vec<f64>, ValidationError> {
        let mut processed = Vec::with_capacity(data.len());

        for (i, &value) in data.iter().enumerate() {
            self.validate_input(value).map_err(|mut e| {
                e.field = format!("data[{}]", i);
                e
            })?;

            let transformed = if value > self.threshold {
                value.ln()
            } else {
                value.sqrt()
            };

            processed.push(transformed);
        }

        Ok(processed)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64), ValidationError> {
        if data.is_empty() {
            return Err(ValidationError {
                field: "data".to_string(),
                message: "Cannot calculate statistics for empty dataset".to_string(),
            });
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;

        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;

        Ok((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_input() {
        let processor = DataProcessor::new(10.0);
        assert!(processor.validate_input(5.0).is_ok());
        assert!(processor.validate_input(0.0).is_ok());
        assert!(processor.validate_input(15.0).is_ok());
    }

    #[test]
    fn test_validation_invalid_input() {
        let processor = DataProcessor::new(10.0);
        assert!(processor.validate_input(-5.0).is_err());
        assert!(processor.validate_input(f64::NAN).is_err());
        assert!(processor.validate_input(f64::INFINITY).is_err());
    }

    #[test]
    fn test_process_data() {
        let processor = DataProcessor::new(10.0);
        let data = vec![4.0, 16.0, 25.0];
        let result = processor.process_data(&data).unwrap();
        
        assert_eq!(result.len(), 3);
        assert!((result[0] - 2.0).abs() < 1e-10);
        assert!((result[1] - 2.772588722239781).abs() < 1e-10);
        assert!((result[2] - 3.2188758248682006).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(10.0);
        let data = vec![2.0, 4.0, 6.0, 8.0];
        let (mean, std_dev) = processor.calculate_statistics(&data).unwrap();
        
        assert!((mean - 5.0).abs() < 1e-10);
        assert!((std_dev - 2.23606797749979).abs() < 1e-10);
    }
}