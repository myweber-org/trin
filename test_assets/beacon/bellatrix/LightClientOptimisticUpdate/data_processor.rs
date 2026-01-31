
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ProcessingError {
    pub message: String,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Processing error: {}", self.message)
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ProcessingError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ProcessingError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }

        Ok(DataProcessor { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        if values.is_empty() {
            return Err(ProcessingError {
                message: "Input values cannot be empty".to_string(),
            });
        }

        let filtered: Vec<f64> = values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .cloned()
            .collect();

        if filtered.is_empty() {
            return Err(ProcessingError {
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

    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64, f64), ProcessingError> {
        let processed = self.process_values(values)?;

        let min = processed
            .iter()
            .fold(f64::INFINITY, |acc, &x| acc.min(x));
        let max = processed
            .iter()
            .fold(f64::NEG_INFINITY, |acc, &x| acc.max(x));
        let avg = processed.iter().sum::<f64>() / processed.len() as f64;

        Ok((min, max, avg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_threshold() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
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
    fn test_empty_input() {
        let processor = DataProcessor::new(0.5).unwrap();
        let result = processor.process_values(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.0).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&values).unwrap();
        assert!(stats.0 > 0.0);
        assert!(stats.1 > 0.0);
        assert!(stats.2 > 0.0);
    }
}