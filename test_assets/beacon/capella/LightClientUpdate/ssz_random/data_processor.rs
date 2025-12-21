
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
        Ok(Self { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: "Input values cannot be empty".to_string(),
            });
        }

        let normalized: Vec<f64> = values
            .iter()
            .map(|&v| {
                if v.is_nan() || v.is_infinite() {
                    0.0
                } else {
                    v.max(0.0)
                }
            })
            .collect();

        let max_value = normalized
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if max_value == 0.0 {
            return Ok(vec![0.0; values.len()]);
        }

        let processed: Vec<f64> = normalized
            .iter()
            .map(|&v| {
                let scaled = v / max_value;
                if scaled >= self.threshold {
                    scaled
                } else {
                    0.0
                }
            })
            .collect();

        Ok(processed)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64, f64), ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: "Cannot calculate statistics for empty dataset".to_string(),
            });
        }

        let valid_values: Vec<f64> = values
            .iter()
            .filter(|&&v| v.is_finite() && !v.is_nan())
            .copied()
            .collect();

        if valid_values.is_empty() {
            return Ok((0.0, 0.0, 0.0));
        }

        let sum: f64 = valid_values.iter().sum();
        let count = valid_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = valid_values
            .iter()
            .map(|&v| {
                let diff = v - mean;
                diff * diff
            })
            .sum::<f64>()
            / count;

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
        let values = vec![0.1, 0.5, 0.8, 0.2];
        let result = processor.process_values(&values);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        assert!(processed[0] == 0.0);
        assert!(processed[1] > 0.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.5).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&values);
        assert!(stats.is_ok());

        let (mean, variance, std_dev) = stats.unwrap();
        assert!((mean - 3.0).abs() < 0.0001);
        assert!((variance - 2.0).abs() < 0.0001);
        assert!((std_dev - 1.4142).abs() < 0.0001);
    }
}