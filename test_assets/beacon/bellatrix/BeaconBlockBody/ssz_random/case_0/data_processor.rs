
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
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
            let processed = self.process_value(value)?;
            results.push(processed);
        }
        
        Ok(results)
    }
    
    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64, f64), ProcessingError> {
        if values.is_empty() {
            return Err(ProcessingError::new("Cannot calculate statistics for empty dataset"));
        }
        
        let processed = self.process_batch(values)?;
        
        let sum: f64 = processed.iter().sum();
        let mean = sum / processed.len() as f64;
        
        let variance: f64 = processed.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / processed.len() as f64;
        
        let max = processed.iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Ok((mean, variance.sqrt(), max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(10.0, 2.0);
        assert!(processor.is_ok());
    }
    
    #[test]
    fn test_invalid_threshold() {
        let processor = DataProcessor::new(0.0, 2.0);
        assert!(processor.is_err());
    }
    
    #[test]
    fn test_process_value_below_threshold() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let result = processor.process_value(5.0);
        assert_eq!(result.unwrap(), 5.0);
    }
    
    #[test]
    fn test_process_value_above_threshold() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let result = processor.process_value(15.0);
        assert_eq!(result.unwrap(), 30.0);
    }
    
    #[test]
    fn test_process_batch() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let values = vec![5.0, 15.0, 8.0, 20.0];
        let result = processor.process_batch(&values).unwrap();
        assert_eq!(result, vec![5.0, 30.0, 8.0, 40.0]);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let values = vec![5.0, 15.0, 8.0];
        let (mean, std_dev, max) = processor.calculate_statistics(&values).unwrap();
        
        let expected_mean = (5.0 + 30.0 + 8.0) / 3.0;
        assert!((mean - expected_mean).abs() < 1e-10);
        assert!(max == 30.0);
    }
}