use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub timestamp: u64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    EmptyValues,
    InvalidTimestamp,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>, timestamp: u64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if values.is_empty() {
            return Err(DataError::EmptyValues);
        }
        if timestamp == 0 {
            return Err(DataError::InvalidTimestamp);
        }

        Ok(Self {
            id,
            values,
            timestamp,
        })
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidId);
        }
        if self.values.is_empty() {
            return Err(DataError::EmptyValues);
        }
        if self.timestamp == 0 {
            return Err(DataError::InvalidTimestamp);
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("sum".to_string(), sum);
        stats.insert("count".to_string(), count);
        stats.insert("variance".to_string(), variance);
        
        if let Some(&min) = self.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("min".to_string(), min);
        }
        
        if let Some(&max) = self.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("max".to_string(), max);
        }
        
        stats
    }
}

pub fn normalize_values(values: &[f64]) -> Result<Vec<f64>, DataError> {
    if values.is_empty() {
        return Err(DataError::EmptyValues);
    }
    
    let min = values.iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .ok_or_else(|| DataError::TransformationError("Cannot find minimum value".to_string()))?;
    
    let max = values.iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .ok_or_else(|| DataError::TransformationError("Cannot find maximum value".to_string()))?;
    
    if (max - min).abs() < f64::EPSILON {
        return Ok(vec![0.5; values.len()]);
    }
    
    let normalized: Vec<f64> = values
        .iter()
        .map(|&x| (x - min) / (max - min))
        .collect();
    
    Ok(normalized)
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<HashMap<String, f64>>, DataError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate()?;
        let stats = record.calculate_statistics();
        results.push(stats);
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0], 1234567890);
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, vec![1.0, 2.0], 1234567890);
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_empty_values() {
        let record = DataRecord::new(1, vec![], 1234567890);
        assert!(matches!(record, Err(DataError::EmptyValues)));
    }

    #[test]
    fn test_normalize_values() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = normalize_values(&values).unwrap();
        assert_eq!(normalized[0], 0.0);
        assert_eq!(normalized[4], 1.0);
    }

    #[test]
    fn test_calculate_statistics() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0, 5.0], 1234567890).unwrap();
        let stats = record.calculate_statistics();
        
        assert_eq!(stats.get("mean"), Some(&3.0));
        assert_eq!(stats.get("sum"), Some(&15.0));
        assert_eq!(stats.get("count"), Some(&5.0));
        assert_eq!(stats.get("min"), Some(&1.0));
        assert_eq!(stats.get("max"), Some(&5.0));
    }
}