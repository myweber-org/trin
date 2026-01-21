
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidTimestamp,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid numeric value"),
            ProcessingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::InvalidValue);
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }

        if record.value > self.threshold {
            return Err(ProcessingError::ValidationFailed(
                format!("Value {} exceeds threshold {}", record.value, self.threshold)
            ));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> DataRecord {
        DataRecord {
            id: record.id,
            value: record.value * 2.0,
            timestamp: record.timestamp + 3600,
        }
    }

    pub fn process_records(&self, records: &[DataRecord]) -> Vec<Result<DataRecord, ProcessingError>> {
        records.iter()
            .map(|record| {
                self.validate_record(record)?;
                Ok(self.transform_record(record))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1609459200,
        };

        let result = processor.process_records(&[record.clone()]);
        assert!(result[0].is_ok());
        
        let processed = result[0].as_ref().unwrap();
        assert_eq!(processed.value, 100.0);
        assert_eq!(processed.timestamp, 1609459200 + 3600);
    }

    #[test]
    fn test_invalid_value() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: f64::NAN,
            timestamp: 1609459200,
        };

        let result = processor.process_records(&[record]);
        assert!(result[0].is_err());
    }

    #[test]
    fn test_threshold_exceeded() {
        let processor = DataProcessor::new(50.0);
        let record = DataRecord {
            id: 1,
            value: 75.0,
            timestamp: 1609459200,
        };

        let result = processor.process_records(&[record]);
        assert!(result[0].is_err());
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
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }

        Ok(DataProcessor { threshold })
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
                message: format!(
                    "No values meet the threshold requirement of {}",
                    self.threshold
                ),
            });
        }

        let mean = filtered.iter().sum::<f64>() / filtered.len() as f64;
        let normalized: Vec<f64> = filtered.iter().map(|&v| v / mean).collect();

        Ok(normalized)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64, f64), ValidationError> {
        if data.len() < 2 {
            return Err(ValidationError {
                message: "Insufficient data for statistics calculation".to_string(),
            });
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data
            .iter()
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / (data.len() - 1) as f64;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());

        let invalid = DataProcessor::new(1.5);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(0.3).unwrap();
        let values = vec![0.1, 0.4, 0.5, 0.2, 0.6];
        let result = processor.process_values(&values);

        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.0).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&data);

        assert!(stats.is_ok());
        let (mean, variance, std_dev) = stats.unwrap();
        assert!((mean - 3.0).abs() < 1e-10);
        assert!((variance - 2.5).abs() < 1e-10);
        assert!((std_dev - 1.58113883).abs() < 1e-6);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        let values: Vec<f64> = records
            .iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|value| value.parse::<f64>().ok())
            .collect();

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_record(&["test".to_string(), "123".to_string()]));
        assert!(!processor.validate_record(&[]));
        assert!(!processor.validate_record(&["".to_string()]));
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&records, 0).unwrap();

        let expected_mean = (10.5 + 15.5 + 12.0) / 3.0;
        assert!((stats.0 - expected_mean).abs() < 0.0001);
    }
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    config: ProcessingConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub require_timestamp: bool,
    pub allowed_metadata_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationError(format!(
                "Too many values: {} > {}",
                record.values.len(),
                self.config.max_values
            )));
        }

        if self.config.require_timestamp && record.timestamp <= 0 {
            return Err(ProcessingError::ValidationError(
                "Invalid timestamp".to_string(),
            ));
        }

        for key in record.metadata.keys() {
            if !self.config.allowed_metadata_keys.contains(key) {
                return Err(ProcessingError::ValidationError(format!(
                    "Disallowed metadata key: {}",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn transform_record(&self, record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        let mut transformed = record.clone();
        
        transformed.values = transformed
            .values
            .iter()
            .map(|&v| {
                if v.is_nan() || v.is_infinite() {
                    0.0
                } else {
                    v
                }
            })
            .collect();

        transformed.metadata.retain(|k, _| {
            self.config.allowed_metadata_keys.contains(k)
        });

        if transformed.values.is_empty() {
            return Err(ProcessingError::TransformationFailed(
                "No valid values after transformation".to_string(),
            ));
        }

        Ok(transformed)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            match self.transform_record(record) {
                Ok(transformed) => results.push(transformed),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);

        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.clone())
            .filter(|&v| v.is_finite())
            .collect();

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let count = all_values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = all_values
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count;
            
            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("min".to_string(), all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
            stats.insert("max".to_string(), all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
        }

        stats
    }
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        ProcessingConfig {
            max_values: 100,
            require_timestamp: true,
            allowed_metadata_keys: vec![
                "source".to_string(),
                "version".to_string(),
                "quality".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: {
                let mut map = HashMap::new();
                map.insert("source".to_string(), "test".to_string());
                map
            },
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_too_many_values() {
        let mut config = ProcessingConfig::default();
        config.max_values = 3;
        let processor = DataProcessor::new(config);
        
        let mut record = create_test_record();
        record.values = vec![1.0; 10];
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let mut record = create_test_record();
        record.values.push(f64::NAN);
        
        let result = processor.transform_record(record);
        assert!(result.is_ok());
        
        let transformed = result.unwrap();
        assert_eq!(transformed.values.len(), 6);
        assert!(transformed.values.iter().all(|&v| v.is_finite()));
    }

    #[test]
    fn test_batch_process() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let records = vec![create_test_record(), create_test_record()];
        
        let result = processor.batch_process(records);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![4.0, 5.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = processor.calculate_statistics(&records);
        assert_eq!(stats.get("total_records").unwrap(), &2.0);
        assert_eq!(stats.get("total_values").unwrap(), &5.0);
        assert_eq!(stats.get("mean").unwrap(), &3.0);
    }
}