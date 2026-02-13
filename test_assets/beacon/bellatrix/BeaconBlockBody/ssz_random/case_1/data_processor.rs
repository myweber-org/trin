
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
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
    
    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut loaded_count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()
                .map_err(|e| format!("Invalid ID at line {}: {}", line_num + 1, e))?;
            
            let value = parts[1].parse::<f64>()
                .map_err(|e| format!("Invalid value at line {}: {}", line_num + 1, e))?;
            
            let record = DataRecord::new(id, value, parts[2])?;
            self.records.push(record);
            loaded_count += 1;
        }
        
        Ok(loaded_count)
    }
    
    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
    
    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }
    
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }
    
    #[test]
    fn test_load_and_process() -> Result<(), Box<dyn Error>> {
        let mut csv_content = NamedTempFile::new()?;
        writeln!(csv_content, "1,10.5,category_a")?;
        writeln!(csv_content, "2,20.0,category_b")?;
        writeln!(csv_content, "3,15.75,category_a")?;
        
        let mut processor = DataProcessor::new();
        let count = processor.load_from_csv(csv_content.path())?;
        
        assert_eq!(count, 3);
        assert_eq!(processor.get_records().len(), 3);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.416666666666666).abs() < 0.0001);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        Ok(())
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".into()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed("Timestamp cannot be negative".into()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationFailed("Values cannot be empty".into()));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) {
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(multiplier);
        record.add_tag("processed".into());
        processed.push(record.clone());
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    for key in records[0].values.keys() {
        let values: Vec<f64> = records.iter()
            .filter_map(|r| r.values.get(key).copied())
            .collect();
        
        if !values.is_empty() {
            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let avg = sum / count;
            
            let variance: f64 = values.iter()
                .map(|v| (v - avg).powi(2))
                .sum::<f64>() / count;
            
            stats.insert(format!("{}_mean", key), avg);
            stats.insert(format!("{}_variance", key), variance);
        }
    }
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: HashMap::from([("temp".into(), 25.5)]),
            tags: vec![],
        };
        
        assert!(record.validate().is_ok());
        
        record.id = 0;
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: HashMap::from([("temp".into(), 25.5)]),
            tags: vec![],
        };
        
        record.transform(2.0);
        assert_eq!(record.values.get("temp"), Some(&51.0));
    }
}