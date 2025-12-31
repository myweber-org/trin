
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

pub struct DataProcessor {
    threshold: f64,
    multiplier: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64, multiplier: f64) -> Result<Self, ProcessingError> {
        if threshold <= 0.0 {
            return Err(ProcessingError::new("Threshold must be positive"));
        }
        if multiplier <= 0.0 {
            return Err(ProcessingError::new("Multiplier must be positive"));
        }
        
        Ok(DataProcessor {
            threshold,
            multiplier,
        })
    }
    
    pub fn process_value(&self, value: f64) -> Result<f64, ProcessingError> {
        if value < 0.0 {
            return Err(ProcessingError::new("Value cannot be negative"));
        }
        
        if value > self.threshold {
            Ok(value * self.multiplier)
        } else {
            Ok(value)
        }
    }
    
    pub fn process_batch(&self, values: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        let mut results = Vec::with_capacity(values.len());
        
        for &value in values {
            match self.process_value(value) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
    
    pub fn get_stats(&self, values: &[f64]) -> Result<(f64, f64, f64), ProcessingError> {
        if values.is_empty() {
            return Err(ProcessingError::new("Cannot compute stats for empty dataset"));
        }
        
        let processed = self.process_batch(values)?;
        
        let sum: f64 = processed.iter().sum();
        let count = processed.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = processed.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_processor_creation() {
        let processor = DataProcessor::new(10.0, 2.0);
        assert!(processor.is_ok());
        
        let invalid = DataProcessor::new(0.0, 2.0);
        assert!(invalid.is_err());
    }
    
    #[test]
    fn test_process_value() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        
        assert_eq!(processor.process_value(5.0).unwrap(), 5.0);
        assert_eq!(processor.process_value(15.0).unwrap(), 30.0);
        
        let result = processor.process_value(-5.0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_process_batch() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let values = vec![5.0, 15.0, 8.0, 20.0];
        
        let processed = processor.process_batch(&values).unwrap();
        assert_eq!(processed, vec![5.0, 30.0, 8.0, 40.0]);
    }
    
    #[test]
    fn test_get_stats() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let values = vec![5.0, 15.0, 8.0, 12.0];
        
        let (mean, variance, std_dev) = processor.get_stats(&values).unwrap();
        
        let expected_mean = (5.0 + 30.0 + 8.0 + 24.0) / 4.0;
        assert!((mean - expected_mean).abs() < 1e-10);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
    }
}