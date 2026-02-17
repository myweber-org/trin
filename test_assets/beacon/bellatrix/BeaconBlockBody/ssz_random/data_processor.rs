use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
            }
        }
        
        self.metadata.insert("source".to_string(), filepath.to_string());
        self.metadata.insert("loaded_at".to_string(), chrono::Local::now().to_rfc3339());
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }
        
        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.data.iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);
        
        stats
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&value| value >= threshold)
            .cloned()
            .collect()
    }

    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn data_summary(&self) -> String {
        format!(
            "Data points: {}, Source: {}",
            self.data.len(),
            self.metadata.get("source").unwrap_or(&"Unknown".to_string())
        )
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
        writeln!(temp_file, "id,value").unwrap();
        writeln!(temp_file, "1,10.5").unwrap();
        writeln!(temp_file, "2,20.3").unwrap();
        writeln!(temp_file, "3,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats["count"], 3.0);
        assert!((stats["mean"] - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_data(15.0);
        assert_eq!(filtered.len(), 2);
    }
}
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: String, value: String) -> &mut Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Values cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count as f64;
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some(DataStatistics {
            count,
            sum,
            mean,
            min,
            max,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataStatistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

pub fn process_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| record.validate().is_ok())
        .filter(|record| {
            if let Some(stats) = record.calculate_statistics() {
                stats.mean.is_finite() && !stats.mean.is_nan()
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

pub fn transform_records(records: &[DataRecord], multiplier: f64) -> Vec<DataRecord> {
    records
        .iter()
        .map(|record| {
            let mut transformed = record.clone();
            transformed.values = record
                .values
                .iter()
                .map(|&v| v * multiplier)
                .collect();
            transformed
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.0);
        
        assert!(record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0).add_value(20.0).add_value(30.0);
        
        let stats = record.calculate_statistics().unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }

    #[test]
    fn test_record_processing() {
        let mut valid_record = DataRecord::new(1, 1234567890);
        valid_record.add_value(1.0);
        
        let mut invalid_record = DataRecord::new(0, 1234567890);
        invalid_record.add_value(1.0);
        
        let records = vec![valid_record, invalid_record];
        let processed = process_records(&records);
        
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].id, 1);
    }
}
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    ValidationError(String),
    #[error("Transformation failed: {0}")]
    TransformationError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputData {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedData {
    pub id: u32,
    pub normalized_value: f64,
    pub processed_at: i64,
    pub is_valid: bool,
}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64) -> Self {
        DataProcessor { min_value, max_value }
    }

    pub fn validate_input(&self, data: &InputData) -> Result<(), DataError> {
        if data.value < self.min_value || data.value > self.max_value {
            return Err(DataError::ValidationError(
                format!("Value {} out of range [{}, {}]", data.value, self.min_value, self.max_value)
            ));
        }
        
        if data.timestamp < 0 {
            return Err(DataError::ValidationError(
                format!("Invalid timestamp: {}", data.timestamp)
            ));
        }
        
        Ok(())
    }

    pub fn process(&self, input: InputData) -> Result<ProcessedData, DataError> {
        self.validate_input(&input)?;
        
        let normalized_value = (input.value - self.min_value) / (self.max_value - self.min_value);
        let processed_at = chrono::Utc::now().timestamp();
        let is_valid = normalized_value >= 0.0 && normalized_value <= 1.0;
        
        Ok(ProcessedData {
            id: input.id,
            normalized_value,
            processed_at,
            is_valid,
        })
    }

    pub fn batch_process(&self, inputs: Vec<InputData>) -> Vec<Result<ProcessedData, DataError>> {
        inputs.into_iter()
            .map(|input| self.process(input))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(0.0, 100.0);
        let input = InputData {
            id: 1,
            value: 50.0,
            timestamp: 1625097600,
        };
        
        assert!(processor.validate_input(&input).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(0.0, 100.0);
        let input = InputData {
            id: 1,
            value: 150.0,
            timestamp: 1625097600,
        };
        
        assert!(processor.validate_input(&input).is_err());
    }

    #[test]
    fn test_process_data() {
        let processor = DataProcessor::new(0.0, 100.0);
        let input = InputData {
            id: 1,
            value: 75.0,
            timestamp: 1625097600,
        };
        
        let result = processor.process(input);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.id, 1);
        assert!((processed.normalized_value - 0.75).abs() < 0.001);
        assert!(processed.is_valid);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut processor = DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        };
        
        processor.register_default_validators();
        processor.register_default_transformers();
        
        processor
    }
    
    fn register_default_validators(&mut self) {
        self.validators.insert(
            "email".to_string(),
            Box::new(|s: &str| s.contains('@') && s.contains('.')),
        );
        
        self.validators.insert(
            "numeric".to_string(),
            Box::new(|s: &str| s.parse::<f64>().is_ok()),
        );
        
        self.validators.insert(
            "not_empty".to_string(),
            Box::new(|s: &str| !s.trim().is_empty()),
        );
    }
    
    fn register_default_transformers(&mut self) {
        self.transformers.insert(
            "uppercase".to_string(),
            Box::new(|s: String| s.to_uppercase()),
        );
        
        self.transformers.insert(
            "trim".to_string(),
            Box::new(|s: String| s.trim().to_string()),
        );
        
        self.transformers.insert(
            "reverse".to_string(),
            Box::new(|s: String| s.chars().rev().collect()),
        );
    }
    
    pub fn validate(&self, validator_name: &str, data: &str) -> bool {
        self.validators
            .get(validator_name)
            .map(|validator| validator(data))
            .unwrap_or(false)
    }
    
    pub fn transform(&self, transformer_name: &str, data: String) -> String {
        self.transformers
            .get(transformer_name)
            .map(|transformer| transformer(data))
            .unwrap_or(data)
    }
    
    pub fn process_pipeline(&self, data: &str, validators: &[&str], transformers: &[&str]) -> Option<String> {
        for validator in validators {
            if !self.validate(validator, data) {
                return None;
            }
        }
        
        let mut result = data.to_string();
        for transformer in transformers {
            result = self.transform(transformer, result);
        }
        
        Some(result)
    }
}

pub fn create_processor() -> DataProcessor {
    DataProcessor::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_validation() {
        let processor = create_processor();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }
    
    #[test]
    fn test_numeric_validation() {
        let processor = create_processor();
        assert!(processor.validate("numeric", "123.45"));
        assert!(!processor.validate("numeric", "abc"));
    }
    
    #[test]
    fn test_transformation_pipeline() {
        let processor = create_processor();
        let result = processor.process_pipeline(
            "  hello  ",
            &["not_empty"],
            &["trim", "uppercase"]
        );
        
        assert_eq!(result, Some("HELLO".to_string()));
    }
    
    #[test]
    fn test_reverse_transformation() {
        let processor = create_processor();
        let result = processor.transform("reverse", "rust".to_string());
        assert_eq!(result, "tsur");
    }
}