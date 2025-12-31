
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub category: String,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(DataError::ValidationError(
                "Value must be a finite number".to_string(),
            ));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }
        
        if self.category.trim().is_empty() {
            return Err(DataError::ValidationError(
                "Category cannot be empty".to_string(),
            ));
        }
        
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let transformed = transform_record(record)?;
        processed.push(transformed);
    }
    
    Ok(processed)
}

fn transform_record(mut record: DataRecord) -> Result<DataRecord, DataError> {
    if record.value < 0.0 {
        record.value = record.value.abs();
    }
    
    record.category = record.category.to_uppercase();
    
    if record.timestamp == 0 {
        record.timestamp = chrono::Utc::now().timestamp();
    }
    
    Ok(record)
}

pub fn calculate_statistics(records: &[DataRecord]) -> Option<Statistics> {
    if records.is_empty() {
        return None;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>()
        / count;
    
    Some(Statistics {
        count: records.len(),
        sum,
        mean,
        variance,
        std_dev: variance.sqrt(),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
            category: "test".to_string(),
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            value: f64::NAN,
            timestamp: -1,
            category: "".to_string(),
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                value: -10.5,
                timestamp: 0,
                category: "alpha".to_string(),
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 1000,
                category: "beta".to_string(),
            },
        ];
        
        let result = process_records(records);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed[0].value, 10.5);
        assert_eq!(processed[0].category, "ALPHA");
        assert!(processed[0].timestamp > 0);
        assert_eq!(processed[1].category, "BETA");
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1,
                category: "test".to_string(),
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 2,
                category: "test".to_string(),
            },
            DataRecord {
                id: 3,
                value: 30.0,
                timestamp: 3,
                category: "test".to_string(),
            },
        ];
        
        let stats = calculate_statistics(&records).unwrap();
        
        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 60.0);
        assert_eq!(stats.mean, 20.0);
        assert!(stats.variance > 0.0);
        assert!(stats.std_dev > 0.0);
    }
}