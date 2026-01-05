
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.record_count(), 3);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        let (mean, variance, std_dev) = processor.calculate_statistics();
        assert!((mean - 15.5).abs() < 0.1);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: String) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
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
            if parts.len() != 4 {
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

            let record = DataRecord::new(
                id,
                value,
                parts[2].to_string(),
                parts[3].to_string(),
            );

            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
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

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "test".to_string(), "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, -5.0, "".to_string(), "2024-01-01".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        assert_eq!(processor.calculate_average(), None);
    }
}
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_threshold: f64,
    normalization_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, normalization_factor: f64) -> Self {
        DataProcessor {
            validation_threshold,
            normalization_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::InvalidData("Empty values vector".to_string()));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData(
                    "Invalid numeric value detected".to_string(),
                ));
            }
        }

        if record.id == 0 {
            return Err(ProcessingError::ValidationFailed("Invalid record ID".to_string()));
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(record)?;

        for value in &mut record.values {
            *value = (*value * self.normalization_factor).abs();
            
            if *value > self.validation_threshold {
                return Err(ProcessingError::TransformationError(
                    "Value exceeds threshold after transformation".to_string(),
                ));
            }
        }

        record.metadata.insert(
            "processed".to_string(),
            "true".to_string(),
        );
        record.metadata.insert(
            "normalization_factor".to_string(),
            self.normalization_factor.to_string(),
        );

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> Result<HashMap<String, f64>, ProcessingError> {
        if records.is_empty() {
            return Err(ProcessingError::InvalidData("No records provided".to_string()));
        }

        let mut stats = HashMap::new();
        let mut total_values = 0;
        let mut sum = 0.0;
        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for record in records {
            self.validate_record(record)?;
            
            for value in &record.values {
                total_values += 1;
                sum += value;
                min = min.min(*value);
                max = max.max(*value);
            }
        }

        let mean = if total_values > 0 { sum / total_values as f64 } else { 0.0 };

        stats.insert("mean".to_string(), mean);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_record() {
        let processor = DataProcessor::new(100.0, 1.0);
        let record = DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_record() {
        let processor = DataProcessor::new(100.0, 1.0);
        let record = DataRecord {
            id: 0,
            values: vec![10.0, f64::NAN, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new(50.0, 2.0);
        let mut record = DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values, vec![20.0, 40.0, 60.0]);
        assert_eq!(record.metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(100.0, 1.0);
        let records = vec![
            DataRecord {
                id: 1,
                values: vec![10.0, 20.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                values: vec![30.0, 40.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = processor.calculate_statistics(&records).unwrap();
        
        assert_eq!(stats.get("mean"), Some(&25.0));
        assert_eq!(stats.get("min"), Some(&10.0));
        assert_eq!(stats.get("max"), Some(&40.0));
        assert_eq!(stats.get("total_records"), Some(&2.0));
        assert_eq!(stats.get("total_values"), Some(&4.0));
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold <= 0.0 {
            return Err(ValidationError {
                message: "Threshold must be positive".to_string(),
            });
        }
        Ok(Self { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: "Input values cannot be empty".to_string(),
            });
        }

        let filtered: Vec<f64> = values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .cloned()
            .collect();

        if filtered.is_empty() {
            return Err(ValidationError {
                message: "No values meet the threshold requirement".to_string(),
            });
        }

        let normalized = self.normalize_data(&filtered);
        Ok(normalized)
    }

    fn normalize_data(&self, values: &[f64]) -> Vec<f64> {
        let max_value = values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        values
            .iter()
            .map(|&v| v / max_value)
            .collect()
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = DataProcessor::new(5.0);
        assert!(processor.is_ok());

        let invalid_processor = DataProcessor::new(0.0);
        assert!(invalid_processor.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(2.0).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_values(&values);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        assert!(!processed.contains(&1.0));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(1.0).unwrap();
        let values = vec![2.0, 4.0, 6.0, 8.0];
        
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert_eq!(mean, 5.0);
        assert_eq!(variance, 5.0);
        assert!((std_dev - 2.236067977).abs() < 1e-9);
    }
}