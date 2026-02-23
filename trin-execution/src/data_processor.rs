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

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let max_value = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);
    
    (average, max_value, count)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    EmptyTags,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::EmptyTags => write!(f, "Record must have at least one tag"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    if record.name.trim().is_empty() {
        return Err(ValidationError::EmptyName);
    }
    if record.value < 0.0 {
        return Err(ValidationError::NegativeValue);
    }
    if record.tags.is_empty() {
        return Err(ValidationError::EmptyTags);
    }
    Ok(())
}

pub fn transform_records(records: Vec<DataRecord>) -> HashMap<String, Vec<DataRecord>> {
    let mut grouped = HashMap::new();
    
    for record in records {
        if let Ok(()) = validate_record(&record) {
            let key = record.tags[0].clone();
            grouped.entry(key).or_insert_with(Vec::new).push(record);
        }
    }
    
    grouped
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record() {
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            tags: vec!["tag1".to_string()],
        };
        
        assert!(validate_record(&valid_record).is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -5.0,
            tags: vec![],
        };
        
        assert!(validate_record(&invalid_record).is_err());
    }
    
    #[test]
    fn test_transform_records() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record1".to_string(),
                value: 10.0,
                tags: vec!["alpha".to_string()],
            },
            DataRecord {
                id: 2,
                name: "Record2".to_string(),
                value: 20.0,
                tags: vec!["beta".to_string()],
            },
            DataRecord {
                id: 3,
                name: "Record3".to_string(),
                value: 30.0,
                tags: vec!["alpha".to_string()],
            },
        ];
        
        let grouped = transform_records(records);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get("alpha").unwrap().len(), 2);
        assert_eq!(grouped.get("beta").unwrap().len(), 1);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "A".to_string(),
                value: 10.0,
                tags: vec!["test".to_string()],
            },
            DataRecord {
                id: 2,
                name: "B".to_string(),
                value: 20.0,
                tags: vec!["test".to_string()],
            },
            DataRecord {
                id: 3,
                name: "C".to_string(),
                value: 30.0,
                tags: vec!["test".to_string()],
            },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}