
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessingError {
    details: String,
}

impl ProcessingError {
    pub fn new(msg: &str) -> Self {
        ProcessingError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ProcessingError {}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::new("ID cannot be zero"));
        }
        
        if self.value < 0.0 {
            return Err(ProcessingError::new("Value cannot be negative"));
        }
        
        if self.category.is_empty() {
            return Err(ProcessingError::new("Category cannot be empty"));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.category = self.category.to_uppercase();
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::new();
    
    for record in records {
        record.validate()?;
        let mut transformed = DataRecord {
            id: record.id,
            value: record.value,
            category: record.category.clone(),
        };
        transformed.transform(2.5);
        processed.push(transformed);
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
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            value: 10.5,
            category: "test".to_string(),
        };
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            value: -5.0,
            category: "".to_string(),
        };
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 42,
            value: 10.0,
            category: "sample".to_string(),
        };
        
        record.transform(3.0);
        assert_eq!(record.value, 30.0);
        assert_eq!(record.category, "SAMPLE");
    }
    
    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, value: 20.0, category: "B".to_string() },
            DataRecord { id: 3, value: 30.0, category: "C".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert!((mean - 20.0).abs() < 0.001);
        assert!((variance - 66.666).abs() < 0.001);
        assert!((std_dev - 8.1649).abs() < 0.001);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_csv_data(input_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    (mean, variance.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,active").unwrap();
        writeln!(file, "1,Test1,10.5,true").unwrap();
        writeln!(file, "2,Test2,-3.2,false").unwrap();
        
        let result = process_csv_data(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
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

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut loaded_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();

            let record = DataRecord::new(id, value, category);
            if record.is_valid() {
                self.records.push(record);
                loaded_count += 1;
            }
        }

        Ok(loaded_count)
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

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "test".to_string());
        assert!(valid_record.is_valid());

        let invalid_id = DataRecord::new(0, 10.5, "test".to_string());
        assert!(!invalid_id.is_valid());

        let invalid_value = DataRecord::new(1, -5.0, "test".to_string());
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(1, 10.5, "".to_string());
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_load_from_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "invalid,data,row").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "B".to_string()));
        processor.records.push(DataRecord::new(3, 30.0, "A".to_string()));

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert_eq!(average.unwrap(), 20.0);
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub category: String,
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub valid_records: Vec<DataRecord>,
    pub invalid_records: Vec<DataRecord>,
    pub statistics: ProcessingStats,
}

#[derive(Debug)]
pub struct ProcessingStats {
    pub total_processed: usize,
    pub average_value: f64,
    pub min_timestamp: i64,
    pub max_timestamp: i64,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(ValidationError::InvalidValue);
        }
        
        if self.timestamp <= 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        if self.category.trim().is_empty() {
            return Err(ValidationError::EmptyCategory);
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
    EmptyCategory,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be non-zero"),
            ValidationError::InvalidValue => write!(f, "Value must be a valid number"),
            ValidationError::InvalidTimestamp => write!(f, "Timestamp must be positive"),
            ValidationError::EmptyCategory => write!(f, "Category cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

pub fn process_records(records: Vec<DataRecord>) -> ProcessingResult {
    let mut valid_records = Vec::new();
    let mut invalid_records = Vec::new();
    let mut total_value = 0.0;
    let mut min_timestamp = i64::MAX;
    let mut max_timestamp = i64::MIN;
    
    for record in records {
        match record.validate() {
            Ok(_) => {
                total_value += record.value;
                min_timestamp = min_timestamp.min(record.timestamp);
                max_timestamp = max_timestamp.max(record.timestamp);
                valid_records.push(record);
            }
            Err(_) => {
                invalid_records.push(record);
            }
        }
    }
    
    let average_value = if !valid_records.is_empty() {
        total_value / valid_records.len() as f64
    } else {
        0.0
    };
    
    let statistics = ProcessingStats {
        total_processed: valid_records.len() + invalid_records.len(),
        average_value,
        min_timestamp: if min_timestamp == i64::MAX { 0 } else { min_timestamp },
        max_timestamp: if max_timestamp == i64::MIN { 0 } else { max_timestamp },
    };
    
    ProcessingResult {
        valid_records,
        invalid_records,
        statistics,
    }
}

pub fn transform_records(records: &[DataRecord], multiplier: f64) -> Vec<DataRecord> {
    records
        .iter()
        .map(|record| DataRecord {
            id: record.id,
            value: record.value * multiplier,
            timestamp: record.timestamp,
            category: record.category.clone(),
        })
        .collect()
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
            category: "test".to_string(),
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1234567890,
            category: "test".to_string(),
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1000,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 2000,
                category: "B".to_string(),
            },
            DataRecord {
                id: 0,
                value: 30.0,
                timestamp: 3000,
                category: "C".to_string(),
            },
        ];
        
        let result = process_records(records);
        
        assert_eq!(result.valid_records.len(), 2);
        assert_eq!(result.invalid_records.len(), 1);
        assert_eq!(result.statistics.average_value, 15.0);
        assert_eq!(result.statistics.min_timestamp, 1000);
        assert_eq!(result.statistics.max_timestamp, 2000);
    }
    
    #[test]
    fn test_transform_records() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1000,
                category: "test".to_string(),
            },
        ];
        
        let transformed = transform_records(&records, 2.0);
        
        assert_eq!(transformed[0].value, 20.0);
        assert_eq!(transformed[0].id, 1);
    }
}