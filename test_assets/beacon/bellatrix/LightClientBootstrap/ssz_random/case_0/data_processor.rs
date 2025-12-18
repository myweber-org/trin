
use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        Self {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationError("Empty values vector".to_string()));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData("Invalid numeric value".to_string()));
            }
            
            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationError(
                    format!("Value {} exceeds threshold {}", value, self.validation_threshold)
                ));
            }
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidData("Negative timestamp".to_string()));
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(record)?;

        for value in &mut record.values {
            *value *= self.transformation_factor;
            
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::TransformationFailed(
                    "Transformation produced invalid value".to_string()
                ));
            }
        }

        record.metadata.insert(
            "processed".to_string(),
            "true".to_string()
        );

        record.metadata.insert(
            "transformation_factor".to_string(),
            self.transformation_factor.to_string()
        );

        Ok(())
    }

    pub fn process_batch(&self, records: &mut [DataRecord]) -> Vec<Result<DataRecord, ProcessingError>> {
        records.iter_mut()
            .map(|record| {
                let mut processed_record = record.clone();
                match self.transform_values(&mut processed_record) {
                    Ok(_) => Ok(processed_record),
                    Err(e) => Err(e),
                }
            })
            .collect()
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let all_values: Vec<f64> = records.iter()
        .flat_map(|r| r.values.iter())
        .cloned()
        .collect();

    let sum: f64 = all_values.iter().sum();
    let mean = sum / total_values as f64;
    
    let variance: f64 = all_values.iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / total_values as f64;
    
    let std_dev = variance.sqrt();

    stats.insert("mean".to_string(), mean);
    stats.insert("std_dev".to_string(), std_dev);
    stats.insert("min".to_string(), all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
    stats.insert("max".to_string(), all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
    stats.insert("total_records".to_string(), records.len() as f64);
    stats.insert("total_values".to_string(), total_values as f64);

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_record() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 20.3, 30.7],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_exceeds_threshold() {
        let processor = DataProcessor::new(100.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![150.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_values() {
        let processor = DataProcessor::new(1000.0, 2.5);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![2.0, 4.0, 6.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values, vec![5.0, 10.0, 15.0]);
        assert_eq!(record.metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![4.0, 5.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = calculate_statistics(&records);
        
        assert_eq!(stats.get("mean"), Some(&3.0));
        assert_eq!(stats.get("total_records"), Some(&2.0));
        assert_eq!(stats.get("total_values"), Some(&5.0));
    }
}