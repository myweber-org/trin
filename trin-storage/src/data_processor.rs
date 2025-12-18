
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyValues,
    InvalidValue(f64),
    MissingMetadata(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "Record ID must be greater than zero"),
            ValidationError::EmptyValues => write!(f, "Record must contain at least one value"),
            ValidationError::InvalidValue(val) => write!(f, "Invalid value detected: {}", val),
            ValidationError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.values.is_empty() {
        return Err(ValidationError::EmptyValues);
    }
    
    for &value in &record.values {
        if value.is_nan() || value.is_infinite() {
            return Err(ValidationError::InvalidValue(value));
        }
    }
    
    Ok(())
}

pub fn normalize_values(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    
    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = max - min;
    
    if range.abs() < f64::EPSILON {
        return vec![0.5; values.len()];
    }
    
    values.iter()
        .map(|&v| (v - min) / range)
        .collect()
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ValidationError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for mut record in records {
        validate_record(&record)?;
        
        let normalized = normalize_values(&record.values);
        record.values = normalized;
        
        processed.push(record);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let all_values: Vec<f64> = records.iter()
        .flat_map(|r| r.values.iter())
        .copied()
        .collect();
    
    let sum: f64 = all_values.iter().sum();
    let count = all_values.len() as f64;
    
    if count > 0.0 {
        let mean = sum / count;
        let variance: f64 = all_values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
        stats.insert("min".to_string(), all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
        stats.insert("max".to_string(), all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
    }
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_id() {
        let record = DataRecord {
            id: 0,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_normalize_values() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = normalize_values(&values);
        
        assert_eq!(normalized.len(), 5);
        assert!((normalized[0] - 0.0).abs() < f64::EPSILON);
        assert!((normalized[4] - 1.0).abs() < f64::EPSILON);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                values: vec![3.0, 4.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = calculate_statistics(&records);
        
        assert_eq!(stats.get("mean").unwrap(), &2.5);
        assert_eq!(stats.get("total_records").unwrap(), &2.0);
        assert_eq!(stats.get("total_values").unwrap(), &4.0);
    }
}