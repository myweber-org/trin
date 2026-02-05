
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ProcessingError {
    message: String,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Processing error: {}", self.message)
    }
}

impl Error for ProcessingError {}

impl ProcessingError {
    pub fn new(msg: &str) -> Self {
        ProcessingError {
            message: msg.to_string(),
        }
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, ProcessingError> {
        if id == 0 {
            return Err(ProcessingError::new("ID cannot be zero"));
        }
        if !value.is_finite() {
            return Err(ProcessingError::new("Value must be finite"));
        }
        if timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }

        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(ProcessingError::new("Value out of valid range (0-1000)"));
        }
        Ok(())
    }
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<f64>, ProcessingError> {
    if records.is_empty() {
        return Err(ProcessingError::new("No records to process"));
    }

    let mut results = Vec::with_capacity(records.len());
    for record in records {
        record.validate()?;
        let processed_value = transform_value(record.value)?;
        results.push(processed_value);
    }

    Ok(results)
}

fn transform_value(value: f64) -> Result<f64, ProcessingError> {
    if value <= 0.0 {
        return Err(ProcessingError::new("Value must be positive for transformation"));
    }

    let transformed = (value * 2.5).ln() / (value + 1.0).sqrt();
    if transformed.is_nan() || transformed.is_infinite() {
        return Err(ProcessingError::new("Transformation produced invalid result"));
    }

    Ok(transformed)
}

pub fn calculate_statistics(values: &[f64]) -> Result<(f64, f64, f64), ProcessingError> {
    if values.is_empty() {
        return Err(ProcessingError::new("Cannot calculate statistics for empty dataset"));
    }

    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;

    let variance: f64 = values
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;

    let std_dev = variance.sqrt();

    Ok((mean, variance, std_dev))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 50.5, 1234567890).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 50.5);
        assert_eq!(record.timestamp, 1234567890);
    }

    #[test]
    fn test_invalid_record_creation() {
        assert!(DataRecord::new(0, 50.5, 1234567890).is_err());
        assert!(DataRecord::new(1, f64::INFINITY, 1234567890).is_err());
        assert!(DataRecord::new(1, 50.5, -1).is_err());
    }

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 500.0, 1234567890).unwrap();
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(1, 1500.0, 1234567890).unwrap();
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 100.0, 1234567890).unwrap(),
            DataRecord::new(2, 200.0, 1234567891).unwrap(),
        ];

        let results = process_records(&records).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].is_finite());
        assert!(results[1].is_finite());
    }

    #[test]
    fn test_calculate_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = calculate_statistics(&values).unwrap();

        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: u64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
    TransformationError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid value field"),
            DataError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            DataError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: u64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if !value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        if timestamp == 0 {
            return Err(DataError::InvalidTimestamp);
        }

        Ok(Self {
            id,
            value,
            timestamp,
        })
    }

    pub fn transform(&self, factor: f64) -> Result<Self, DataError> {
        if !factor.is_finite() || factor <= 0.0 {
            return Err(DataError::TransformationError(
                "Invalid transformation factor".to_string(),
            ));
        }

        let transformed_value = self.value * factor;
        Ok(Self {
            id: self.id,
            value: transformed_value,
            timestamp: self.timestamp,
        })
    }

    pub fn normalize(&self, max_value: f64) -> Result<f64, DataError> {
        if max_value <= 0.0 || !max_value.is_finite() {
            return Err(DataError::TransformationError(
                "Invalid max value for normalization".to_string(),
            ));
        }

        if self.value > max_value {
            return Err(DataError::TransformationError(
                "Value exceeds maximum allowed".to_string(),
            ));
        }

        Ok(self.value / max_value)
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<DataRecord, DataError>> {
    records
        .iter()
        .map(|record| record.transform(2.0))
        .collect()
}

pub fn validate_record_batch(records: &[DataRecord]) -> Result<(), DataError> {
    for record in records {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        if !record.value.is_finite() {
            return Err(DataError::InvalidValue);
        }
        if record.timestamp == 0 {
            return Err(DataError::InvalidTimestamp);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1234567890);
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 42.5, 1234567890);
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord::new(1, 10.0, 1234567890).unwrap();
        let transformed = record.transform(2.0);
        assert!(transformed.is_ok());
        assert_eq!(transformed.unwrap().value, 20.0);
    }

    #[test]
    fn test_normalize_record() {
        let record = DataRecord::new(1, 50.0, 1234567890).unwrap();
        let normalized = record.normalize(100.0);
        assert!(normalized.is_ok());
        assert_eq!(normalized.unwrap(), 0.5);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    config: ProcessingConfig,
}

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
                "Record contains {} values, maximum allowed is {}",
                record.values.len(),
                self.config.max_values
            )));
        }

        if self.config.require_timestamp && record.timestamp <= 0 {
            return Err(ProcessingError::ValidationError(
                "Timestamp must be positive".to_string(),
            ));
        }

        for key in record.metadata.keys() {
            if !self.config.allowed_metadata_keys.contains(key) {
                return Err(ProcessingError::ValidationError(format!(
                    "Metadata key '{}' is not allowed",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::TransformationError(
                "No values to transform".to_string(),
            ));
        }

        let sum: f64 = record.values.iter().sum();
        let count = record.values.len() as f64;
        
        if count == 0.0 {
            return Err(ProcessingError::TransformationError(
                "Cannot calculate average of empty values".to_string(),
            ));
        }

        let average = sum / count;
        
        record.values = record
            .values
            .iter()
            .map(|&value| (value - average).abs())
            .collect();

        record.metadata.insert(
            "transformed".to_string(),
            "true".to_string(),
        );
        record.metadata.insert(
            "original_average".to_string(),
            format!("{:.4}", average),
        );

        Ok(())
    }

    pub fn process_batch(&self, records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::new();
        
        for record in records.iter_mut() {
            self.validate_record(record)?;
            self.transform_values(record)?;
            processed.push(record.clone());
        }
        
        Ok(processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata,
        }
    }

    #[test]
    fn test_validation_success() {
        let config = ProcessingConfig {
            max_values: 10,
            require_timestamp: true,
            allowed_metadata_keys: vec!["source".to_string()],
        };
        
        let processor = DataProcessor::new(config);
        let record = create_test_record();
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let config = ProcessingConfig {
            max_values: 3,
            require_timestamp: true,
            allowed_metadata_keys: vec!["source".to_string()],
        };
        
        let processor = DataProcessor::new(config);
        let record = create_test_record();
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_values() {
        let config = ProcessingConfig {
            max_values: 10,
            require_timestamp: true,
            allowed_metadata_keys: vec!["source".to_string()],
        };
        
        let processor = DataProcessor::new(config);
        let mut record = create_test_record();
        
        assert!(processor.transform_values(&mut record).is_ok());
        assert_eq!(record.values.len(), 5);
        assert!(record.metadata.contains_key("transformed"));
        assert!(record.metadata.contains_key("original_average"));
    }
}
use std::collections::HashMap;

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

        let processed = Self::normalize_data(data)?;
        let transformed = Self::apply_transformations(&processed);
        
        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn normalize_data(data: &[f64]) -> Result<Vec<f64>, String> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        
        if variance.abs() < 1e-10 {
            return Err("Zero variance detected".to_string());
        }

        let std_dev = variance.sqrt();
        Ok(data.iter().map(|&x| (x - mean) / std_dev).collect())
    }

    fn apply_transformations(data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).sin().abs())
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
    fn test_normalize_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = DataProcessor::normalize_data(&data).unwrap();
        
        let mean = normalized.iter().sum::<f64>() / normalized.len() as f64;
        let variance: f64 = normalized.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / normalized.len() as f64;
        
        assert!(mean.abs() < 1e-10);
        assert!((variance - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        assert!(result.is_err());
    }
}