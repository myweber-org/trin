
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

    pub fn process_numeric_data(&mut self, key: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if data.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }

        let processed: Vec<f64> = data
            .iter()
            .map(|&x| x * 2.0)
            .filter(|&x| x > 0.0)
            .collect();

        if processed.is_empty() {
            return Err("All values filtered out".to_string());
        }

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn get_cached_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn calculate_statistics(data: &[f64]) -> (f64, f64, f64) {
        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_valid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        let result = processor.process_numeric_data("test", data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_process_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![f64::NAN, 1.0];
        let result = processor.process_numeric_data("test", data);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = DataProcessor::calculate_statistics(&data);
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
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

    pub fn process(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut processed_record = record.clone();

        for rule in &self.validation_rules {
            rule.validate(&processed_record)?;
        }

        for transformation in &self.transformation_pipeline {
            processed_record = transformation.apply(&processed_record)?;
        }

        Ok(processed_record)
    }

    pub fn batch_process(
        &self,
        records: &[DataRecord],
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        records
            .iter()
            .map(|record| self.process(record))
            .collect()
    }
}

pub trait ValidationRule {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError>;
}

pub trait Transformation {
    fn apply(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError>;
}

pub struct RequiredFieldValidator {
    field_name: String,
}

impl RequiredFieldValidator {
    pub fn new(field_name: &str) -> Self {
        RequiredFieldValidator {
            field_name: field_name.to_string(),
        }
    }
}

impl ValidationRule for RequiredFieldValidator {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if !record.values.contains_key(&self.field_name) {
            return Err(ProcessingError::MissingField(self.field_name.clone()));
        }
        Ok(())
    }
}

pub struct ValueNormalizer {
    target_field: String,
    scale_factor: f64,
}

impl ValueNormalizer {
    pub fn new(target_field: &str, scale_factor: f64) -> Self {
        ValueNormalizer {
            target_field: target_field.to_string(),
            scale_factor,
        }
    }
}

impl Transformation for ValueNormalizer {
    fn apply(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut new_record = record.clone();
        
        if let Some(value) = new_record.values.get_mut(&self.target_field) {
            *value *= self.scale_factor;
        }
        
        Ok(new_record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(RequiredFieldValidator::new("temperature"));
        processor.add_transformation(ValueNormalizer::new("temperature", 0.556));

        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 100.0);
        values.insert("humidity".to_string(), 50.0);

        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values,
            tags: vec!["sensor".to_string(), "room1".to_string()],
        };

        let result = processor.process(&record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!((processed.values["temperature"] - 55.6).abs() < 0.001);
    }
}use std::error::Error;
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
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>], expected_columns: usize) -> Vec<usize> {
        let mut invalid_rows = Vec::new();
        
        for (index, row) in records.iter().enumerate() {
            if row.len() != expected_columns {
                invalid_rows.push(index);
            }
        }
        
        invalid_rows
    }
}

pub fn calculate_average(numbers: &[f64]) -> Option<f64> {
    if numbers.is_empty() {
        return None;
    }
    
    let sum: f64 = numbers.iter().sum();
    Some(sum / numbers.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["x".to_string(), "y".to_string()],
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records, 3);
        
        assert_eq!(invalid, vec![1]);
    }

    #[test]
    fn test_calculate_average() {
        assert_eq!(calculate_average(&[1.0, 2.0, 3.0]), Some(2.0));
        assert_eq!(calculate_average(&[]), None);
    }
}
use csv::{Reader, Writer};
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

impl Record {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut wtr = Writer::from_path(output_path)?;

    for result in rdr.deserialize() {
        let mut record: Record = result?;
        
        if let Err(e) = record.validate() {
            eprintln!("Validation error for record {}: {}", record.id, e);
            continue;
        }

        record.value *= 1.1;
        wtr.serialize(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            active: true,
        };
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -50.0,
            active: false,
        };
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: false },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}