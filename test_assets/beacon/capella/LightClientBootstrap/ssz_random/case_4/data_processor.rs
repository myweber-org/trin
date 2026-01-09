
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

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::new("ID cannot be zero"));
        }
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(ProcessingError::new("Value must be a finite number"));
        }
        if self.timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }
        Ok(())
    }

    pub fn normalize(&mut self, factor: f64) -> Result<(), ProcessingError> {
        if factor == 0.0 {
            return Err(ProcessingError::new("Normalization factor cannot be zero"));
        }
        self.value /= factor;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.normalize(factor)?;
        
        if record.value > 1000.0 {
            return Err(ProcessingError::new("Normalized value exceeds maximum threshold"));
        }
        
        processed.push(record.clone());
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
        };
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1234567890,
        };
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord {
            id: 1,
            value: 100.0,
            timestamp: 1234567890,
        };
        assert!(record.normalize(10.0).is_ok());
        assert_eq!(record.value, 10.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1 },
            DataRecord { id: 2, value: 20.0, timestamp: 2 },
            DataRecord { id: 3, value: 30.0, timestamp: 3 },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ProcessedData {
    pub record: DataRecord,
    pub statistics: DataStatistics,
    pub is_valid: bool,
}

#[derive(Debug)]
pub struct DataStatistics {
    pub mean: f64,
    pub median: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
}

pub fn validate_record(record: &DataRecord) -> Result<(), Box<dyn Error>> {
    if record.id == 0 {
        return Err("Invalid record ID".into());
    }
    
    if record.timestamp <= 0 {
        return Err("Invalid timestamp".into());
    }
    
    if record.values.is_empty() {
        return Err("Empty values array".into());
    }
    
    for value in &record.values {
        if !value.is_finite() {
            return Err("Non-finite value detected".into());
        }
    }
    
    Ok(())
}

pub fn calculate_statistics(values: &[f64]) -> DataStatistics {
    let mut sorted_values = values.to_vec();
    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;
    
    let median = if values.len() % 2 == 0 {
        let mid = values.len() / 2;
        (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
    } else {
        sorted_values[values.len() / 2]
    };
    
    let variance = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    
    let min = *sorted_values.first().unwrap_or(&0.0);
    let max = *sorted_values.last().unwrap_or(&0.0);
    
    DataStatistics {
        mean,
        median,
        variance,
        min,
        max,
    }
}

pub fn process_data_record(record: DataRecord) -> Result<ProcessedData, Box<dyn Error>> {
    validate_record(&record)?;
    
    let statistics = calculate_statistics(&record.values);
    let is_valid = statistics.variance < 1000.0 && record.values.len() >= 3;
    
    Ok(ProcessedData {
        record,
        statistics,
        is_valid,
    })
}

pub fn transform_values(values: &[f64], factor: f64) -> Vec<f64> {
    values.iter()
        .map(|&x| x * factor)
        .collect()
}

pub fn filter_valid_records(records: Vec<DataRecord>) -> Vec<ProcessedData> {
    records.into_iter()
        .filter_map(|record| process_data_record(record).ok())
        .filter(|processed| processed.is_valid)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record() {
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(validate_record(&valid_record).is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(validate_record(&invalid_record).is_err());
    }
    
    #[test]
    fn test_calculate_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = calculate_statistics(&values);
        
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }
    
    #[test]
    fn test_transform_values() {
        let values = vec![1.0, 2.0, 3.0];
        let transformed = transform_values(&values, 2.0);
        
        assert_eq!(transformed, vec![2.0, 4.0, 6.0]);
    }
}