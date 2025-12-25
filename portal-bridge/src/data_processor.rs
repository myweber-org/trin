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

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value < 0.0 || record.name.is_empty())
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, f64, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let min = self
            .records
            .iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);
        let max = self
            .records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        (count, avg, min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let csv_data = "id,name,value,category\n\
                       1,ItemA,10.5,Alpha\n\
                       2,ItemB,-5.0,Beta\n\
                       3,ItemC,15.75,Alpha\n\
                       4,,20.0,Gamma";

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let alpha_records = processor.filter_by_category("Alpha");
        assert_eq!(alpha_records.len(), 2);

        let invalid_records = processor.validate_records();
        assert_eq!(invalid_records.len(), 2);

        let stats = processor.get_statistics();
        assert_eq!(stats.0, 4);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
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
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            metadata: None,
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        if let Some(map) = self.metadata.as_mut() {
            map.insert(key.to_string(), value.to_string());
        }
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError(
                "Record ID cannot be zero".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(ProcessingError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record must contain at least one value".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(ProcessingError::ValidationError(
                    "Value key cannot be empty".to_string(),
                ));
            }
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationError(format!(
                    "Invalid numeric value for key '{}'",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) -> Result<(), ProcessingError>
    where
        F: Fn(f64) -> Result<f64, String>,
    {
        let mut transformed = HashMap::new();
        
        for (key, value) in &self.values {
            match transform_fn(*value) {
                Ok(transformed_value) => {
                    transformed.insert(key.clone(), transformed_value);
                }
                Err(err) => {
                    return Err(ProcessingError::TransformationFailed(format!(
                        "Failed to transform value for key '{}': {}",
                        key, err
                    )));
                }
            }
        }
        
        self.values = transformed;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        
        record.transform_values(|value| {
            if value < 0.0 {
                Err("Negative values are not allowed".to_string())
            } else {
                Ok(value * 2.0)
            }
        })?;
        
        processed.push(record.clone());
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, (f64, f64, f64)> {
    let mut stats = HashMap::new();
    
    for record in records {
        for (key, value) in &record.values {
            let entry = stats.entry(key.clone()).or_insert((0.0, 0.0, 0.0));
            entry.0 += value;
            entry.1 = entry.1.max(*value);
            entry.2 = if entry.2 == 0.0 {
                *value
            } else {
                entry.2.min(*value)
            };
        }
    }
    
    for (_, (sum, max, min)) in stats.iter_mut() {
        *sum = (*sum / records.len() as f64).round();
    }
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut valid_record = DataRecord::new(1, 1625097600);
        valid_record.add_value("temperature", 25.5);
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1625097600);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value("score", 10.5);
        
        let result = record.transform_values(|value| Ok(value * 2.0));
        assert!(result.is_ok());
        assert_eq!(record.values.get("score"), Some(&21.0));
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            {
                let mut r = DataRecord::new(1, 1625097600);
                r.add_value("value", 5.0);
                r
            },
            {
                let mut r = DataRecord::new(2, 1625097601);
                r.add_value("value", 10.0);
                r
            },
        ];
        
        let result = process_records(&mut records);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].values.get("value"), Some(&10.0));
        assert_eq!(processed[1].values.get("value"), Some(&20.0));
    }
}