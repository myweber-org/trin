
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
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
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
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

            let category = parts[2].to_string();

            let record = DataRecord::new(id, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
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

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
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

        let invalid_record = DataRecord::new(0, -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,type_a").unwrap();
        writeln!(temp_file, "2,15.3,type_b").unwrap();
        writeln!(temp_file, "3,invalid,type_c").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.records.len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "a".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "a".to_string()));
        processor.records.push(DataRecord::new(3, 30.0, "b".to_string()));

        let stats = processor.get_statistics();
        assert_eq!(stats, (10.0, 30.0, 20.0));
        
        let filtered = processor.filter_by_category("a");
        assert_eq!(filtered.len(), 2);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct DataProcessor {
    validation_rules: HashMap<String, ValidationRule>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_validation_rule(&mut self, field: String, rule: ValidationRule) {
        self.validation_rules.insert(field, rule);
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<ProcessedRecord, ProcessingError> {
        self.validate_record(record)?;
        
        let transformed_values = self.transform_values(&record.values);
        let computed_metrics = self.compute_metrics(&transformed_values);
        
        Ok(ProcessedRecord {
            original_id: record.id,
            processed_timestamp: chrono::Utc::now().timestamp(),
            transformed_values,
            computed_metrics,
            validation_passed: true,
        })
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::InvalidFormat);
        }

        for (field, rule) in &self.validation_rules {
            if let Some(value) = record.values.get(field) {
                if !rule.is_valid(*value) {
                    return Err(ProcessingError::OutOfRange(
                        format!("Field '{}' failed validation", field)
                    ));
                }
            } else if rule.required {
                return Err(ProcessingError::MissingField(field.clone()));
            }
        }

        Ok(())
    }

    fn transform_values(&self, values: &HashMap<String, f64>) -> HashMap<String, f64> {
        values.iter()
            .map(|(key, value)| {
                let transformed = if key.starts_with("log_") {
                    value.ln()
                } else if key.starts_with("norm_") {
                    *value / 100.0
                } else {
                    *value
                };
                (key.clone(), transformed)
            })
            .collect()
    }

    fn compute_metrics(&self, values: &HashMap<String, f64>) -> Metrics {
        let count = values.len() as f64;
        let sum: f64 = values.values().sum();
        let mean = if count > 0.0 { sum / count } else { 0.0 };
        
        let variance: f64 = values.values()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count.max(1.0);
        
        Metrics {
            count: values.len(),
            sum,
            mean,
            variance,
            min: values.values().cloned().fold(f64::INFINITY, f64::min),
            max: values.values().cloned().fold(f64::NEG_INFINITY, f64::max),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub required: bool,
}

impl ValidationRule {
    pub fn new(min: Option<f64>, max: Option<f64>, required: bool) -> Self {
        Self { min, max, required }
    }

    pub fn is_valid(&self, value: f64) -> bool {
        if let Some(min) = self.min {
            if value < min {
                return false;
            }
        }
        
        if let Some(max) = self.max {
            if value > max {
                return false;
            }
        }
        
        true
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessedRecord {
    pub original_id: u64,
    pub processed_timestamp: i64,
    pub transformed_values: HashMap<String, f64>,
    pub computed_metrics: Metrics,
    pub validation_passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metrics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(
            "temperature".to_string(),
            ValidationRule::new(Some(-50.0), Some(100.0), true)
        );

        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 25.5);
        values.insert("humidity".to_string(), 65.0);

        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values,
            metadata: None,
        };

        let result = processor.process_record(&record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.original_id, 1);
        assert!(processed.validation_passed);
        assert_eq!(processed.computed_metrics.count, 2);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(
            "pressure".to_string(),
            ValidationRule::new(Some(900.0), Some(1100.0), true)
        );

        let mut values = HashMap::new();
        values.insert("pressure".to_string(), 1200.0);

        let record = DataRecord {
            id: 2,
            timestamp: 1234567890,
            values,
            metadata: None,
        };

        let result = processor.process_record(&record);
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
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
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].trim();

            let record = DataRecord::new(id, value, category);
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
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
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "A");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "A");
        assert!(record.valid);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "");
        assert!(!record.valid);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,42.5,TypeA").unwrap();
        writeln!(temp_file, "2,15.3,TypeB").unwrap();
        writeln!(temp_file, "3,-5.0,TypeC").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A"));
        processor.records.push(DataRecord::new(2, 20.0, "B"));
        processor.records.push(DataRecord::new(3, -5.0, "C"));

        let average = processor.calculate_average();
        assert_eq!(average, Some(15.0));
    }

    #[test]
    fn test_empty_average() {
        let processor = DataProcessor::new();
        let average = processor.calculate_average();
        assert_eq!(average, None);
    }
}