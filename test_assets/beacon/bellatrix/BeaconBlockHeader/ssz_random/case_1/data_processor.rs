
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
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        if data.is_empty() {
            return None;
        }

        let mut values = Vec::new();
        for record in data {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
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
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 3);

        let stats = processor.calculate_statistics(&data, 2);
        assert!(stats.is_some());
        let (mean, _, std_dev) = stats.unwrap();
        assert!((mean - 50000.0).abs() < 0.01);
        assert!(std_dev > 0.0);
    }

    #[test]
    fn test_invalid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "Alice,30").unwrap();
        writeln!(temp_file, ",25").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        assert!(result.is_err());
    }
}use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x < rule.min_value || x > rule.max_value) {
                return Err(format!("Validation failed for field: {}", rule.field_name));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&value| {
                let transformed = value * 2.5 - 1.0;
                transformed.max(0.0)
            })
            .collect();

        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStats> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&value| {
                    let diff = mean - value;
                    diff * diff
                })
                .sum::<f64>() / count;

            DatasetStats {
                mean,
                variance,
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                count: data.len(),
            }
        })
    }
}

pub struct DatasetStats {
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("temperature", -50.0, 100.0, true);
        processor.add_validation_rule(rule);

        let data = vec![10.0, 20.0, 30.0, 40.0];
        let result = processor.process_dataset("test_data", &data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("test_data").unwrap().len(), 4);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("pressure", 0.0, 10.0, true);
        processor.add_validation_rule(rule);

        let invalid_data = vec![5.0, 15.0, 8.0];
        let result = processor.process_dataset("invalid", &invalid_data);
        
        assert!(result.is_err());
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: ValidationRules,
}

pub struct ValidationRules {
    min_value: f64,
    max_value: f64,
    required_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(rules: ValidationRules) -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if !self.validation_rules.required_keys.contains(&key) {
            return Err(format!("Key '{}' is not in required keys list", key));
        }

        for &value in &values {
            if value < self.validation_rules.min_value || value > self.validation_rules.max_value {
                return Err(format!("Value {} is outside allowed range [{}, {}]", 
                    value, self.validation_rules.min_value, self.validation_rules.max_value));
            }
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            let std_dev = variance.sqrt();

            Statistics {
                count,
                sum,
                mean,
                variance,
                std_dev,
            }
        })
    }

    pub fn normalize_data(&mut self, key: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(key) {
            let stats = self.calculate_statistics(key).unwrap();
            
            for value in values {
                *value = (*value - stats.mean) / stats.std_dev;
            }
            Ok(())
        } else {
            Err(format!("Key '{}' not found in dataset", key))
        }
    }

    pub fn merge_datasets(&mut self, other: DataProcessor) {
        for (key, values) in other.data {
            self.data.entry(key)
                .and_modify(|existing| existing.extend_from_slice(&values))
                .or_insert(values);
        }
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl ValidationRules {
    pub fn new(min_value: f64, max_value: f64, required_keys: Vec<String>) -> Self {
        ValidationRules {
            min_value,
            max_value,
            required_keys,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let rules = ValidationRules::new(
            0.0,
            100.0,
            vec!["temperature".to_string(), "humidity".to_string()]
        );
        
        let mut processor = DataProcessor::new(rules);
        
        assert!(processor.add_dataset("temperature".to_string(), vec![25.5, 30.0, 22.8]).is_ok());
        assert!(processor.add_dataset("invalid_key".to_string(), vec![50.0]).is_err());
        assert!(processor.add_dataset("temperature".to_string(), vec![150.0]).is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let rules = ValidationRules::new(f64::MIN, f64::MAX, vec!["test".to_string()]);
        let mut processor = DataProcessor::new(rules);
        
        processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("test").unwrap();
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.sum, 15.0);
        assert_eq!(stats.mean, 3.0);
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

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_rules: Vec<ValidationRule>,
    transformation_pipeline: Vec<Transformation>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformation_pipeline.push(transformation);
    }

    pub fn process(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            rule.validate(record)?;
        }

        for transformation in &self.transformation_pipeline {
            transformation.apply(record)?;
        }

        Ok(())
    }

    pub fn batch_process(&self, records: &mut [DataRecord]) -> Vec<Result<(), ProcessingError>> {
        records
            .iter_mut()
            .map(|record| self.process(record))
            .collect()
    }
}

pub trait ValidationRule {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError>;
}

pub trait Transformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError>;
}

pub struct TimestampValidator {
    min_timestamp: i64,
    max_timestamp: i64,
}

impl TimestampValidator {
    pub fn new(min: i64, max: i64) -> Self {
        TimestampValidator {
            min_timestamp: min,
            max_timestamp: max,
        }
    }
}

impl ValidationRule for TimestampValidator {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.timestamp < self.min_timestamp || record.timestamp > self.max_timestamp {
            Err(ProcessingError::ValidationFailed(format!(
                "Timestamp {} out of range [{}, {}]",
                record.timestamp, self.min_timestamp, self.max_timestamp
            )))
        } else {
            Ok(())
        }
    }
}

pub struct NormalizationTransformation {
    factor: f64,
}

impl NormalizationTransformation {
    pub fn new(factor: f64) -> Self {
        NormalizationTransformation { factor }
    }
}

impl Transformation for NormalizationTransformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if self.factor == 0.0 {
            return Err(ProcessingError::TransformationError(
                "Normalization factor cannot be zero".to_string(),
            ));
        }

        for value in record.values.iter_mut() {
            *value /= self.factor;
        }

        Ok(())
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let sum: f64 = records.iter().flat_map(|r| r.values.iter()).sum();
    let mean = sum / total_values as f64;

    let variance: f64 = records
        .iter()
        .flat_map(|r| r.values.iter())
        .map(|v| (v - mean).powi(2))
        .sum::<f64>()
        / total_values as f64;

    stats.insert("mean".to_string(), mean);
    stats.insert("variance".to_string(), variance);
    stats.insert("total_records".to_string(), records.len() as f64);
    stats.insert("total_values".to_string(), total_values as f64);

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(Box::new(TimestampValidator::new(0, 1000)));
        processor.add_transformation(Box::new(NormalizationTransformation::new(10.0)));

        let mut record = DataRecord {
            id: 1,
            timestamp: 500,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(&mut record);
        assert!(result.is_ok());
        assert_eq!(record.values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 100,
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 200,
                values: vec![4.0, 5.0, 6.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = calculate_statistics(&records);
        assert_eq!(stats["mean"], 3.5);
        assert_eq!(stats["total_records"], 2.0);
    }
}