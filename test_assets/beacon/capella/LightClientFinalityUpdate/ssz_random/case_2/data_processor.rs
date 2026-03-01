
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
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
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let mut count = 0;
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 3 {
                if let (Ok(id), Ok(value), timestamp) = (
                    parts[0].parse::<u32>(),
                    parts[1].parse::<f64>(),
                    parts[2].to_string(),
                ) {
                    self.records.push(DataRecord {
                        id,
                        value,
                        timestamp,
                    });
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "id,value,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,2023-01-01T12:00:00").unwrap();
        writeln!(temp_file, "2,20.3,2023-01-01T12:05:00").unwrap();
        writeln!(temp_file, "3,15.7,2023-01-01T12:10:00").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.record_count(), 3);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.01);

        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error for field '{}': {}", self.field, self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> Result<(), ValidationError>>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_validator<F>(&mut self, field: &str, validator: F)
    where
        F: Fn(&str) -> Result<(), ValidationError> + 'static,
    {
        self.validators.insert(field.to_string(), Box::new(validator));
    }

    pub fn add_transformer<F>(&mut self, field: &str, transformer: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformers.insert(field.to_string(), Box::new(transformer));
    }

    pub fn process_data(&self, data: &mut HashMap<String, String>) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (field, value) in data.iter() {
            if let Some(validator) = self.validators.get(field) {
                if let Err(err) = validator(value) {
                    errors.push(err);
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        for (field, value) in data.iter_mut() {
            if let Some(transformer) = self.transformers.get(field) {
                *value = transformer(value.clone());
            }
        }

        Ok(())
    }
}

pub fn create_email_validator() -> impl Fn(&str) -> Result<(), ValidationError> {
    move |email: &str| {
        if email.contains('@') && email.contains('.') {
            Ok(())
        } else {
            Err(ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            })
        }
    }
}

pub fn create_name_transformer() -> impl Fn(String) -> String {
    move |name: String| {
        let mut chars = name.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = create_email_validator();
        assert!(validator("test@example.com").is_ok());
        assert!(validator("invalid-email").is_err());
    }

    #[test]
    fn test_name_transformation() {
        let transformer = create_name_transformer();
        assert_eq!(transformer("john".to_string()), "John");
        assert_eq!(transformer("JANE".to_string()), "JANE");
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validator("email", create_email_validator());
        processor.add_transformer("name", create_name_transformer());

        let mut data = HashMap::new();
        data.insert("email".to_string(), "test@example.com".to_string());
        data.insert("name".to_string(), "john doe".to_string());

        let result = processor.process_data(&mut data);
        assert!(result.is_ok());
        assert_eq!(data.get("name").unwrap(), "John doe");
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
pub enum DataError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Record ID must be greater than 0"),
            DataError::EmptyValues => write!(f, "Record must contain at least one value"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            DataError::MissingMetadata(key) => write!(f, "Required metadata '{}' is missing", key),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>, metadata: HashMap<String, String>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if values.is_empty() {
            return Err(DataError::EmptyValues);
        }
        
        for &value in &values {
            if !value.is_finite() {
                return Err(DataError::ValueOutOfRange(value));
            }
        }
        
        Ok(Self { id, values, metadata })
    }
    
    pub fn validate_metadata(&self, required_keys: &[&str]) -> Result<(), DataError> {
        for &key in required_keys {
            if !self.metadata.contains_key(key) {
                return Err(DataError::MissingMetadata(key.to_string()));
            }
        }
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
    
    pub fn normalize_values(&mut self) {
        let (mean, _, std_dev) = self.calculate_statistics();
        
        if std_dev > 0.0 {
            for value in &mut self.values {
                *value = (*value - mean) / std_dev;
            }
        }
    }
}

pub fn process_records(records: &mut [DataRecord], required_metadata: &[&str]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate_metadata(required_metadata)?;
        let mut processed_record = record.clone();
        processed_record.normalize_values();
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn aggregate_records(records: &[DataRecord]) -> HashMap<u32, (f64, f64)> {
    let mut aggregates = HashMap::new();
    
    for record in records {
        let (mean, _, std_dev) = record.calculate_statistics();
        aggregates.insert(record.id, (mean, std_dev));
    }
    
    aggregates
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "sensor_a".to_string());
        metadata.insert("timestamp".to_string(), "2024-01-15T10:30:00Z".to_string());
        
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let record = DataRecord::new(1, values, metadata).unwrap();
        
        assert_eq!(record.id, 1);
        assert_eq!(record.values.len(), 5);
        assert_eq!(record.metadata.get("source").unwrap(), "sensor_a");
    }
    
    #[test]
    fn test_invalid_id() {
        let metadata = HashMap::new();
        let values = vec![1.0, 2.0];
        
        let result = DataRecord::new(0, values, metadata);
        assert!(matches!(result, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_statistics_calculation() {
        let metadata = HashMap::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let record = DataRecord::new(1, values, metadata).unwrap();
        
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    ValueOutOfRange(f64, f64),
}

pub struct DataProcessor {
    value_range: (f64, f64),
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        DataProcessor {
            value_range: (min_value, max_value),
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.timestamp < 0 {
            return Err(ValidationError::InvalidTimestamp);
        }

        if record.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.value_range.0 || value > self.value_range.1 {
                return Err(ValidationError::ValueOutOfRange(
                    self.value_range.0,
                    self.value_range.1,
                ));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        let min_val = record
            .values
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let max_val = record
            .values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);

        if max_val > min_val {
            for value in &mut record.values {
                *value = (*value - min_val) / (max_val - min_val);
            }
        }
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: Vec<f64> = records.iter().flat_map(|r| r.values.clone()).collect();

        let sum: f64 = total_values.iter().sum();
        let count = total_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = total_values
            .iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>()
            / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), total_values.iter().copied().fold(f64::INFINITY, f64::min));
        stats.insert("max".to_string(), total_values.iter().copied().fold(f64::NEG_INFINITY, f64::max));
        stats.insert("count".to_string(), count);

        stats
    }

    pub fn filter_by_metadata(
        &self,
        records: &[DataRecord],
        key: &str,
        value: &str,
    ) -> Vec<DataRecord> {
        records
            .iter()
            .filter(|record| record.metadata.get(key).map_or(false, |v| v == value))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_record() {
        let processor = DataProcessor::new(0.0, 100.0);
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "sensor_a".to_string());

        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_id() {
        let processor = DataProcessor::new(0.0, 100.0);
        let record = DataRecord {
            id: 0,
            timestamp: 1625097600,
            values: vec![10.0],
            metadata: HashMap::new(),
        };

        assert_eq!(processor.validate_record(&record), Err(ValidationError::InvalidId));
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(0.0, 100.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        processor.normalize_values(&mut record);
        assert_eq!(record.values, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0, 100.0);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1625097600,
                values: vec![10.0, 20.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1625097601,
                values: vec![30.0, 40.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = processor.calculate_statistics(&records);
        assert_eq!(stats.get("mean"), Some(&25.0));
        assert_eq!(stats.get("count"), Some(&4.0));
    }

    #[test]
    fn test_filter_by_metadata() {
        let processor = DataProcessor::new(0.0, 100.0);
        let mut metadata1 = HashMap::new();
        metadata1.insert("type".to_string(), "temperature".to_string());

        let mut metadata2 = HashMap::new();
        metadata2.insert("type".to_string(), "humidity".to_string());

        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1625097600,
                values: vec![10.0],
                metadata: metadata1,
            },
            DataRecord {
                id: 2,
                timestamp: 1625097601,
                values: vec![20.0],
                metadata: metadata2,
            },
        ];

        let filtered = processor.filter_by_metadata(&records, "type", "temperature");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
}