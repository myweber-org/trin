
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
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub total_value: f64,
    pub record_count: usize,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        self.records.push(record.clone());
        self.update_category_stats(&record);
        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string(),
            ));
        }

        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError(
                "Record value must be non-negative".to_string(),
            ));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record category cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                total_value: 0.0,
                record_count: 0,
                average_value: 0.0,
            });

        stats.total_value += record.value;
        stats.record_count += 1;
        stats.average_value = stats.total_value / stats.record_count as f64;
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) -> Result<(), ProcessingError>
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            let original_value = record.value;
            record.value = transform_fn(record.value);
            
            if record.value.is_nan() || record.value.is_infinite() {
                return Err(ProcessingError::TransformationFailed(
                    format!("Transformation produced invalid value for record {}", record.id)
                ));
            }
            
            if record.value < 0.0 {
                record.value = original_value;
                return Err(ProcessingError::TransformationFailed(
                    format!("Transformation produced negative value for record {}", record.id)
                ));
            }
        }
        
        self.recalculate_all_stats();
        Ok(())
    }

    fn recalculate_all_stats(&mut self) {
        self.category_stats.clear();
        
        for record in &self.records {
            self.update_category_stats(record);
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.get_total_value() / self.records.len() as f64
        }
    }

    pub fn export_records(&self) -> Vec<String> {
        self.records
            .iter()
            .map(|record| {
                format!("{},{},{:.2},{}", record.id, record.name, record.value, record.category)
            })
            .collect()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 50.0,
            category: "CategoryA".to_string(),
        };

        let record2 = DataRecord {
            id: 2,
            name: "Record 2".to_string(),
            value: 150.0,
            category: "CategoryA".to_string(),
        };

        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();

        let stats = processor.get_category_stats("CategoryA").unwrap();
        assert_eq!(stats.total_value, 200.0);
        assert_eq!(stats.record_count, 2);
        assert_eq!(stats.average_value, 100.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "Test".to_string(),
        };

        processor.add_record(record).unwrap();
        
        let result = processor.transform_values(|x| x * 2.0);
        assert!(result.is_ok());
        assert_eq!(processor.records[0].value, 20.0);
    }
}
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

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

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
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = Reader::from_reader(file);
        
        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = if count > 0.0 { sum / count } else { 0.0 };
        
        let variance: f64 = if count > 0.0 {
            values.iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count
        } else {
            0.0
        };
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
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
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_record_count(), 3);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        let (mean, variance, std_dev) = processor.calculate_statistics();
        assert!((mean - 15.5).abs() < 0.1);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
    }
}