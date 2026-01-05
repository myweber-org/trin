
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Result<Self, ProcessingError> {
        if values.is_empty() {
            return Err(ProcessingError::InvalidData("Values cannot be empty".to_string()));
        }
        
        if values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
            return Err(ProcessingError::InvalidData("Values contain NaN or infinite numbers".to_string()));
        }
        
        Ok(Self {
            id,
            values,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.values.len() > 1000 {
            return Err(ProcessingError::ValidationError("Too many values".to_string()));
        }
        
        Ok(())
    }
    
    pub fn normalize(&mut self) -> Result<(), ProcessingError> {
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return Err(ProcessingError::TransformationError("Cannot normalize constant values".to_string()));
        }
        
        for value in &mut self.values {
            *value = (*value - min) / (max - min);
        }
        
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), *self.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        stats.insert("max".to_string(), *self.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        stats.insert("sum".to_string(), sum);
        
        stats
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }
    
    pub fn process_all(&mut self) -> Result<(), ProcessingError> {
        for record in &mut self.records {
            record.normalize()?;
        }
        Ok(())
    }
    
    pub fn get_aggregated_stats(&self) -> HashMap<String, f64> {
        let mut aggregated = HashMap::new();
        let mut total_mean = 0.0;
        let mut total_variance = 0.0;
        
        for record in &self.records {
            let stats = record.calculate_statistics();
            total_mean += stats.get("mean").unwrap_or(&0.0);
            total_variance += stats.get("variance").unwrap_or(&0.0);
        }
        
        let count = self.records.len() as f64;
        aggregated.insert("average_mean".to_string(), total_mean / count);
        aggregated.insert("average_variance".to_string(), total_variance / count);
        aggregated.insert("total_records".to_string(), count);
        
        aggregated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.values.len(), 3);
    }
    
    #[test]
    fn test_record_validation() {
        let record = DataRecord::new(0, vec![1.0]).unwrap();
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        record.normalize().unwrap();
        assert_eq!(record.values[0], 0.0);
        assert_eq!(record.values[2], 1.0);
    }
    
    #[test]
    fn test_statistics() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        let stats = record.calculate_statistics();
        assert_eq!(stats.get("mean").unwrap(), &2.0);
        assert_eq!(stats.get("sum").unwrap(), &6.0);
    }
}