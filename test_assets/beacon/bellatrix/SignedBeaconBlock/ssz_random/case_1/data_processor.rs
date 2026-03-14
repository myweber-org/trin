use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            if let Some(value) = record.get(0) {
                if let Ok(num) = value.parse::<f64>() {
                    self.data.push(num);
                }
            }
        }
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn filter_outliers(&mut self, threshold: f64) {
        if let (Some(mean), Some(std_dev)) = (self.calculate_mean(), self.calculate_standard_deviation()) {
            self.data.retain(|&x| (x - mean).abs() <= threshold * std_dev);
        }
    }

    pub fn get_data(&self) -> &[f64] {
        &self.data
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "value\n10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        assert!(processor.load_from_csv(temp_file.path()).is_ok());
        assert_eq!(processor.get_data().len(), 5);
        
        let mean = processor.calculate_mean().unwrap();
        assert!(mean > 15.0 && mean < 20.0);
        
        let original_len = processor.get_data().len();
        processor.filter_outliers(2.0);
        assert!(processor.get_data().len() <= original_len);
        
        processor.clear();
        assert!(processor.get_data().is_empty());
    }
}
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

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());

        for record in records {
            match self.process(record) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }
}

fn validate_timestamp(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.timestamp < 0 {
        Err(ProcessingError::ValidationFailed(
            "Timestamp cannot be negative".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn normalize_values(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    if record.values.is_empty() {
        return Err(ProcessingError::TransformationError(
            "Empty values array".to_string(),
        ));
    }

    let sum: f64 = record.values.iter().sum();
    if sum == 0.0 {
        return Err(ProcessingError::TransformationError(
            "Sum of values is zero".to_string(),
        ));
    }

    let normalized_values: Vec<f64> = record
        .values
        .iter()
        .map(|&v| v / sum)
        .collect();

    Ok(DataRecord {
        values: normalized_values,
        ..record
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(validate_timestamp);
        processor.add_transformation(normalize_values);

        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(result.is_ok());

        let processed = result.unwrap();
        let sum: f64 = processed.values.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(validate_timestamp);

        let record = DataRecord {
            id: 2,
            timestamp: -100,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub name: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();
            
            if value < 0.0 {
                return Err(format!("Negative value at line {}", line_num + 1).into());
            }
            
            self.records.push(DataRecord {
                id,
                name,
                value,
                category,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.calculate_total() / self.records.len() as f64)
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn record_count(&self) -> usize {
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,Category1").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        assert_eq!(processor.calculate_total(), 46.5);
        
        let avg = processor.average_value();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.001);
        
        let category1_items = processor.filter_by_category("Category1");
        assert_eq!(category1_items.len(), 2);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);
    }
}
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid data value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ProcessingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value.is_nan() || record.value.is_infinite() {
        return Err(ProcessingError::InvalidValue);
    }
    
    if record.timestamp < 0 {
        return Err(ProcessingError::InvalidTimestamp);
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> DataRecord {
    DataRecord {
        id: record.id,
        value: record.value * 2.0,
        timestamp: record.timestamp + 3600,
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed_records = Vec::with_capacity(records.len());
    
    for record in records {
        validate_record(&record)?;
        let transformed = transform_record(&record);
        processed_records.push(transformed);
    }
    
    Ok(processed_records)
}

pub fn serialize_to_json(records: &[DataRecord]) -> Result<String, ProcessingError> {
    serde_json::to_string(records)
        .map_err(|e| ProcessingError::SerializationError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1625097600,
        };
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_record_invalid_value() {
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1625097600,
        };
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            value: 10.0,
            timestamp: 1000,
        };
        let transformed = transform_record(&record);
        assert_eq!(transformed.value, 20.0);
        assert_eq!(transformed.timestamp, 4600);
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 5.0,
                timestamp: 1000,
            },
            DataRecord {
                id: 2,
                value: 15.0,
                timestamp: 2000,
            },
        ];
        
        let result = process_records(records);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].value, 10.0);
        assert_eq!(processed[1].value, 30.0);
    }
}