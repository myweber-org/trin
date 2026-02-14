
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

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();

        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }

        groups
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.records.iter().map(|r| r.value).reduce(f64::max);

        (count, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let csv_data = "id,name,value,category\n1,ItemA,10.5,Alpha\n2,ItemB,15.0,Beta\n3,ItemC,20.3,Alpha";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        
        let valid_records = processor.validate_records();
        assert_eq!(valid_records.len(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.266).abs() < 0.001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.get("Alpha").unwrap().len(), 2);
        assert_eq!(groups.get("Beta").unwrap().len(), 1);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
        assert!((stats.1.unwrap() - 15.266).abs() < 0.001);
        assert!((stats.2.unwrap() - 20.3).abs() < 0.001);
    }
}
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: i64,
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid data value: {0}")]
    InvalidValue(f64),
    #[error("Timestamp out of range: {0}")]
    InvalidTimestamp(i64),
    #[error("ID must be positive: {0}")]
    InvalidId(u32),
}

pub struct DataProcessor {
    max_value: f64,
    min_timestamp: i64,
    max_timestamp: i64,
}

impl DataProcessor {
    pub fn new(max_value: f64, min_timestamp: i64, max_timestamp: i64) -> Self {
        Self {
            max_value,
            min_timestamp,
            max_timestamp,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId(record.id));
        }

        if record.value < 0.0 || record.value > self.max_value {
            return Err(DataError::InvalidValue(record.value));
        }

        if record.timestamp < self.min_timestamp || record.timestamp > self.max_timestamp {
            return Err(DataError::InvalidTimestamp(record.timestamp));
        }

        Ok(())
    }

    pub fn normalize_value(&self, record: &DataRecord) -> f64 {
        record.value / self.max_value
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, DataError>> {
        records
            .into_iter()
            .map(|record| {
                self.validate_record(&record)?;
                Ok(record)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new(100.0, 0, 1000);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 500,
        };

        assert!(processor.validate_record(&record).is_ok());
        assert_eq!(processor.normalize_value(&record), 0.5);
    }

    #[test]
    fn test_invalid_value() {
        let processor = DataProcessor::new(100.0, 0, 1000);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 500,
        };

        assert!(matches!(
            processor.validate_record(&record),
            Err(DataError::InvalidValue(150.0))
        ));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };
            
            let category = parts[2].trim().to_string();
            
            self.records.push(DataRecord {
                id,
                value,
                category,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.find_max_value().map(|r| r.value);
        
        (count, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,type_a").unwrap();
        writeln!(temp_file, "2,15.3,type_b").unwrap();
        writeln!(temp_file, "3,8.7,type_a").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        let type_a_records = processor.filter_by_category("type_a");
        assert_eq!(type_a_records.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 11.5).abs() < 0.001);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);
    }
}use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: i64,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp must be non-negative"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        if timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        Ok(Self { id, value, timestamp })
    }
    
    pub fn transform(&self, multiplier: f64) -> Result<f64, ValidationError> {
        if multiplier <= 0.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        let transformed = self.value * multiplier;
        if transformed > 1000.0 {
            Err(ValidationError::InvalidValue)
        } else {
            Ok(transformed)
        }
    }
    
    pub fn normalize(&self, max_value: f64) -> Result<f64, ValidationError> {
        if max_value <= 0.0 || max_value < self.value {
            return Err(ValidationError::InvalidValue);
        }
        
        Ok(self.value / max_value)
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<f64, ValidationError>> {
    records.iter()
        .map(|record| record.transform(2.0))
        .collect()
}

pub fn validate_batch(records: &[DataRecord]) -> bool {
    records.iter().all(|record| {
        record.id > 0 && 
        record.value >= 0.0 && 
        record.value <= 1000.0 && 
        record.timestamp >= 0
    })
}