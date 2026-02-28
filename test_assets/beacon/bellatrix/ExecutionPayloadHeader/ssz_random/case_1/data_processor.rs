
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.id == 0 {
        return Err(ProcessingError::ValidationFailed("ID cannot be zero".into()));
    }
    
    if record.timestamp < 0 {
        return Err(ProcessingError::ValidationFailed("Timestamp cannot be negative".into()));
    }
    
    if record.values.is_empty() {
        return Err(ProcessingError::ValidationFailed("Values cannot be empty".into()));
    }
    
    Ok(())
}

pub fn normalize_values(record: &mut DataRecord) -> Result<(), ProcessingError> {
    if record.values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
        return Err(ProcessingError::TransformationError("Invalid numeric values".into()));
    }
    
    let min = record.values
        .iter()
        .fold(f64::INFINITY, |a, &b| a.min(b));
    let max = record.values
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    if (max - min).abs() < f64::EPSILON {
        return Err(ProcessingError::TransformationError("Cannot normalize constant values".into()));
    }
    
    for value in &mut record.values {
        *value = (*value - min) / (max - min);
    }
    
    Ok(())
}

pub fn process_records(records: &mut [DataRecord]) -> Vec<Result<DataRecord, ProcessingError>> {
    records
        .iter_mut()
        .map(|record| {
            validate_record(record)
                .and_then(|_| normalize_values(record))
                .map(|_| record.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(validate_record(&record).is_ok());
        assert!(normalize_values(&mut record).is_ok());
        assert_eq!(record.values[0], 0.0);
        assert_eq!(record.values[2], 1.0);
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: vec![1.0],
            metadata: HashMap::new(),
        };
        
        assert!(validate_record(&record).is_err());
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn process_data(input_path: &str, output_path: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= threshold && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

fn filter_records(records: Vec<Record>, predicate: impl Fn(&Record) -> bool) -> Vec<Record> {
    records.into_iter()
        .filter(predicate)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: false },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_filter_function() {
        let records = vec![
            Record { id: 1, name: "X".to_string(), value: 5.0, active: true },
            Record { id: 2, name: "Y".to_string(), value: 15.0, active: false },
            Record { id: 3, name: "Z".to_string(), value: 25.0, active: true },
        ];
        
        let filtered = filter_records(records, |r| r.value > 10.0 && r.active);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 3);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: u64) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
    }
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut loaded_count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();
            let timestamp = parts[3].parse::<u64>().unwrap_or(0);

            let record = DataRecord::new(id, value, category, timestamp);
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

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
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
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test".to_string(), 1234567890);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, 42.5, "test".to_string(), 1234567890);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut csv_content = "id,value,category,timestamp\n".to_string();
        csv_content.push_str("1,100.5,alpha,1000\n");
        csv_content.push_str("2,200.3,beta,2000\n");
        csv_content.push_str("3,300.7,alpha,3000\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());

        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
    }

    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A".to_string(), 1000));
        processor.records.push(DataRecord::new(2, 20.0, "B".to_string(), 2000));
        processor.records.push(DataRecord::new(3, 30.0, "A".to_string(), 3000));

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);

        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
    }
}use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        for &value in data {
            if !value.is_finite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }
        Ok(data.to_vec())
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        if variance.abs() < 1e-10 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / variance.sqrt())
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).ln_1p().tanh())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let test_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_dataset("test", &test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), test_data.len());
        
        let stats = processor.cache_stats();
        assert_eq!(stats.0, 1);
        assert_eq!(stats.1, 5);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_dataset("invalid", &invalid_data);
        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue,
    InvalidTimestamp,
    MissingField,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue => write!(f, "Invalid data value"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::MissingField => write!(f, "Missing required field"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue);
        }

        if record.timestamp < 0 {
            return Err(DataError::InvalidTimestamp);
        }

        Ok(())
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<DataRecord, DataError> {
        self.validate_record(record)?;

        let processed_value = if record.value > self.threshold {
            record.value * 0.9
        } else {
            record.value * 1.1
        };

        Ok(DataRecord {
            id: record.id,
            value: processed_value,
            timestamp: record.timestamp,
        })
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> (Vec<DataRecord>, Vec<(DataRecord, DataError)>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.process_record(&record) {
                Ok(processed_record) => processed.push(processed_record),
                Err(err) => errors.push((record, err)),
            }
        }

        (processed, errors)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> Option<(f64, f64, f64)> {
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

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_record() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1625097600,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_invalid_value() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1625097600,
        };

        assert!(matches!(
            processor.validate_record(&record),
            Err(DataError::InvalidValue)
        ));
    }

    #[test]
    fn test_process_record_above_threshold() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1625097600,
        };

        let result = processor.process_record(&record).unwrap();
        assert_eq!(result.value, 135.0);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(100.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 50.0,
                timestamp: 1625097600,
            },
            DataRecord {
                id: 2,
                value: f64::INFINITY,
                timestamp: 1625097600,
            },
        ];

        let (processed, errors) = processor.batch_process(records);
        assert_eq!(processed.len(), 1);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(100.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1625097600,
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 1625097600,
            },
            DataRecord {
                id: 3,
                value: 30.0,
                timestamp: 1625097600,
            },
        ];

        let stats = processor.calculate_statistics(&records).unwrap();
        assert_eq!(stats.0, 20.0);
    }
}