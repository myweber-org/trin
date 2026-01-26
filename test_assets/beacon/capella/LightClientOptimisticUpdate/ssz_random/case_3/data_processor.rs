
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::InvalidFormat);
        }
        
        if self.timestamp < 0 {
            return Err(ProcessingError::OutOfRange("timestamp".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(ProcessingError::MissingField("values".to_string()));
        }
        
        Ok(())
    }
    
    pub fn normalize_values(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if !self.values.is_empty() {
            let sum: f64 = self.values.iter().sum();
            let count = self.values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = self.values
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("count".to_string(), count);
            stats.insert("sum".to_string(), sum);
            
            if let Some(min) = self.values.iter().copied().reduce(f64::min) {
                stats.insert("min".to_string(), min);
            }
            
            if let Some(max) = self.values.iter().copied().reduce(f64::max) {
                stats.insert("max".to_string(), max);
            }
        }
        
        stats
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<HashMap<String, f64>>, ProcessingError> {
    let mut results = Vec::new();
    
    for record in records {
        record.validate()?;
        record.normalize_values();
        results.push(record.calculate_statistics());
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_normalize_values() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![2.0, 4.0, 6.0],
            metadata: HashMap::new(),
        };
        
        record.normalize_values();
        assert_eq!(record.values, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0],
            metadata: HashMap::new(),
        };
        
        let stats = record.calculate_statistics();
        assert_eq!(stats.get("mean"), Some(&2.5));
        assert_eq!(stats.get("count"), Some(&4.0));
        assert_eq!(stats.get("sum"), Some(&10.0));
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    MissingMetadata(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        Self {
            id,
            name,
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }

    pub fn to_json(&self) -> String {
        let metadata_json: String = self.metadata
            .iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
            .collect::<Vec<String>>()
            .join(",");
        
        format!(
            "{{\"id\":{},\"name\":\"{}\",\"value\":{},\"metadata\":{{{}}}}}",
            self.id, self.name, self.value, metadata_json
        )
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Vec<Result<(), ValidationError>> {
    records
        .iter_mut()
        .map(|record| {
            record.validate()?;
            record.transform_value(multiplier);
            Ok(())
        })
        .collect()
}

pub fn filter_records(records: &[DataRecord], min_value: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.value >= min_value)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, "Test".to_string(), 100.0);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, "Test".to_string(), 100.0);
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_negative_value() {
        let record = DataRecord::new(1, "Test".to_string(), -50.0);
        assert!(matches!(record.validate(), Err(ValidationError::NegativeValue)));
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 100.0);
        record.transform_value(2.5);
        assert_eq!(record.value, 250.0);
    }

    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, "Test".to_string(), 100.0);
        record.add_metadata("category".to_string(), "premium".to_string());
        assert_eq!(record.get_metadata("category"), Some(&"premium".to_string()));
    }
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = Writer::from_path(path)?;
        for record in &self.records {
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn add_record(&mut self, id: u32, name: String, value: f64, active: bool) {
        self.records.push(Record {
            id,
            name,
            value,
            active,
        });
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);

        processor.add_record(1, "Test".to_string(), 10.5, true);
        processor.add_record(2, "Sample".to_string(), 20.0, false);
        
        assert_eq!(processor.get_record_count(), 2);
        assert_eq!(processor.calculate_total(), 30.5);
        
        let active_records = processor.filter_active();
        assert_eq!(active_records.len(), 1);
        assert_eq!(active_records[0].id, 1);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_record(1, "Alpha".to_string(), 15.0, true);
        processor.add_record(2, "Beta".to_string(), 25.0, false);

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();
        
        processor.save_to_csv(path)?;
        
        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path)?;
        
        assert_eq!(new_processor.get_record_count(), 2);
        assert_eq!(new_processor.calculate_total(), 40.0);
        
        Ok(())
    }
}