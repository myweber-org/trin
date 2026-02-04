use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
                
                let category = parts[0].to_string();
                *self.frequency_map.entry(category).or_insert(0) += 1;
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

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mid = self.data.len() / 2;
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
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
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(15.5));
        assert_eq!(processor.calculate_median(), Some(15.7));
        
        let filtered = processor.filter_by_threshold(12.0);
        assert_eq!(filtered.len(), 2);
        
        let freq = processor.get_frequency_distribution();
        assert_eq!(freq.get("A"), Some(&2));
        assert_eq!(freq.get("B"), Some(&1));
    }
}
use csv;
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
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
    
    fn process(&mut self) {
        self.name = self.name.trim().to_uppercase();
        if self.value > 1000.0 {
            self.value = 1000.0;
        }
    }
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }
    
    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }
    
    fn validate_records(&self) -> Vec<&Record> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }
    
    fn process_all(&mut self) {
        for record in &mut self.records {
            record.process();
        }
    }
    
    fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = csv::Writer::from_writer(file);
        
        for record in &self.records {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }
    
    fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len() as f64;
        
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    match processor.load_from_csv("input_data.csv") {
        Ok(_) => println!("Data loaded successfully"),
        Err(e) => eprintln!("Error loading data: {}", e),
    }
    
    let valid_records = processor.validate_records();
    println!("Valid records: {}", valid_records.len());
    
    processor.process_all();
    
    match processor.save_to_csv("processed_data.csv") {
        Ok(_) => println!("Data saved successfully"),
        Err(e) => eprintln!("Error saving data: {}", e),
    }
    
    let (mean, variance, std_dev) = processor.calculate_statistics();
    println!("Statistics - Mean: {:.2}, Variance: {:.2}, Std Dev: {:.2}", 
             mean, variance, std_dev);
    
    Ok(())
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
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Invalid category".to_string());
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_valid_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,A").unwrap();
        writeln!(temp_file, "2,Test2,20.0,B").unwrap();

        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].value, 20.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "C".to_string() },
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_invalid_record_validation() {
        let invalid_record = Record {
            id: 1,
            name: "".to_string(),
            value: -5.0,
            category: "D".to_string(),
        };

        let result = validate_record(&invalid_record);
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let mut records = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|field| field.to_string()).collect();
            
            if Self::validate_row(&row) {
                records.push(row);
            } else {
                eprintln!("Warning: Invalid row skipped");
            }
        }
        
        Ok(records)
    }

    fn validate_row(row: &[String]) -> bool {
        !row.is_empty() && row.iter().all(|field| !field.trim().is_empty())
    }

    pub fn calculate_statistics(data: &[Vec<String>]) -> (usize, usize) {
        let total_rows = data.len();
        let total_fields = data.iter().map(|row| row.len()).sum();
        (total_rows, total_fields)
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "Alice");
        
        let stats = DataProcessor::calculate_statistics(&result);
        assert_eq!(stats, (2, 6));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if let Some(value_str) = parts.get(0) {
                if let Ok(value) = value_str.parse::<f64>() {
                    self.data.push(value);
                }
            }
            
            if let Some(category) = parts.get(1) {
                *self.frequency_map.entry(category.to_string()).or_insert(0) += 1;
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

    pub fn calculate_median(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mid = sorted_data.len() / 2;
        if sorted_data.len() % 2 == 0 {
            Some((sorted_data[mid - 1] + sorted_data[mid]) / 2.0)
        } else {
            Some(sorted_data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let median = self.calculate_median().unwrap_or(0.0);
        let count = self.data.len();
        let unique_categories = self.frequency_map.len();
        
        format!(
            "Data Summary:\n  Count: {}\n  Mean: {:.2}\n  Median: {:.2}\n  Unique Categories: {}",
            count, mean, median, unique_categories
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
        writeln!(temp_file, "10.5,CategoryA").unwrap();
        writeln!(temp_file, "20.3,CategoryB").unwrap();
        writeln!(temp_file, "15.7,CategoryA").unwrap();
        writeln!(temp_file, "25.1,CategoryC").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(17.9));
        assert_eq!(processor.calculate_median(), Some(17.9));
        
        let frequencies = processor.get_frequency_distribution();
        assert_eq!(frequencies.get("CategoryA"), Some(&2));
        assert_eq!(frequencies.get("CategoryB"), Some(&1));
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 3);
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

pub struct RangeValidation {
    pub min_value: f64,
    pub max_value: f64,
    pub value_index: usize,
}

impl ValidationRule for RangeValidation {
    fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if let Some(&value) = record.values.get(self.value_index) {
            if value >= self.min_value && value <= self.max_value {
                Ok(())
            } else {
                Err(ProcessingError::ValidationFailed(format!(
                    "Value {} at index {} is outside range [{}, {}]",
                    value, self.value_index, self.min_value, self.max_value
                )))
            }
        } else {
            Err(ProcessingError::ValidationFailed(format!(
                "Index {} out of bounds for values array",
                self.value_index
            )))
        }
    }
}

pub struct NormalizationTransformation {
    pub mean: f64,
    pub std_dev: f64,
    pub value_index: usize,
}

impl Transformation for NormalizationTransformation {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if let Some(value) = record.values.get_mut(self.value_index) {
            *value = (*value - self.mean) / self.std_dev;
            Ok(())
        } else {
            Err(ProcessingError::TransformationError(format!(
                "Cannot normalize value at index {}: index out of bounds",
                self.value_index
            )))
        }
    }
}

pub struct MetadataEnricher {
    pub key: String,
    pub value: String,
}

impl Transformation for MetadataEnricher {
    fn apply(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        record.metadata.insert(self.key.clone(), self.value.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing_pipeline() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(RangeValidation {
            min_value: 0.0,
            max_value: 100.0,
            value_index: 0,
        });

        processor.add_transformation(NormalizationTransformation {
            mean: 50.0,
            std_dev: 25.0,
            value_index: 0,
        });

        processor.add_transformation(MetadataEnricher {
            key: "processed".to_string(),
            value: "true".to_string(),
        });

        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![75.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(&mut record);
        assert!(result.is_ok());
        assert_eq!(record.values[0], 1.0);
        assert_eq!(record.metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        
        processor.add_validation_rule(RangeValidation {
            min_value: 0.0,
            max_value: 100.0,
            value_index: 0,
        });

        let mut record = DataRecord {
            id: 2,
            timestamp: 1234567890,
            values: vec![150.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(&mut record);
        assert!(result.is_err());
    }
}