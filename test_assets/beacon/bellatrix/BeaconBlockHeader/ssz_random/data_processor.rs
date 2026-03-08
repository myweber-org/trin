
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: HashMap<String, ValidationRule>,
}

pub struct ValidationRule {
    min_value: Option<f64>,
    max_value: Option<f64>,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) {
        self.data.insert(key.to_string(), values);
    }

    pub fn set_validation_rule(&mut self, key: &str, rule: ValidationRule) {
        self.validation_rules.insert(key.to_string(), rule);
    }

    pub fn validate_dataset(&self, key: &str) -> Result<(), String> {
        let data = match self.data.get(key) {
            Some(d) => d,
            None => return Err(format!("Dataset '{}' not found", key)),
        };

        let rule = match self.validation_rules.get(key) {
            Some(r) => r,
            None => return Ok(()),
        };

        if rule.required && data.is_empty() {
            return Err(format!("Dataset '{}' is required but empty", key));
        }

        for &value in data {
            if let Some(min) = rule.min_value {
                if value < min {
                    return Err(format!("Value {} below minimum {} in dataset '{}'", value, min, key));
                }
            }
            
            if let Some(max) = rule.max_value {
                if value > max {
                    return Err(format!("Value {} above maximum {} in dataset '{}'", value, max, key));
                }
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        let data = self.data.get(key)?;
        
        if data.is_empty() {
            return None;
        }

        let sum: f64 = data.iter().sum();
        let count = data.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let mut sorted_data = data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_data[mid - 1] + sorted_data[mid]) / 2.0
        } else {
            sorted_data[count as usize / 2]
        };

        Some(Statistics {
            mean,
            median,
            std_dev,
            min: *sorted_data.first().unwrap(),
            max: *sorted_data.last().unwrap(),
            count: data.len(),
        })
    }

    pub fn normalize_data(&self, key: &str) -> Option<Vec<f64>> {
        let data = self.data.get(key)?;
        
        if data.is_empty() {
            return None;
        }

        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return Some(vec![0.5; data.len()]);
        }

        Some(
            data.iter()
                .map(|&x| (x - min) / (max - min))
                .collect()
        )
    }
}

pub struct Statistics {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl ValidationRule {
    pub fn new() -> Self {
        ValidationRule {
            min_value: None,
            max_value: None,
            required: false,
        }
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    pub fn with_max(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if fields.len() < 2 {
                return Err("Invalid CSV format: insufficient columns".into());
            }
            
            records.push(fields);
        }

        if records.is_empty() {
            return Err("Empty file provided".into());
        }

        Ok(records)
    }

    pub fn validate_numeric_column(&self, data: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut numeric_values = Vec::new();
        
        for (row_num, record) in data.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds at row {}", column_index, row_num).into());
            }
            
            match record[column_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Non-numeric value found at row {} column {}", row_num, column_index).into()),
            }
        }
        
        Ok(numeric_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_numeric_column() {
        let data = vec![
            vec!["test".to_string(), "42.5".to_string()],
            vec!["test2".to_string(), "18.0".to_string()],
        ];
        
        let processor = DataProcessor::new("dummy.csv");
        let result = processor.validate_numeric_column(&data, 1).unwrap();
        
        assert_eq!(result, vec![42.5, 18.0]);
    }
}use csv::Reader;
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
        return Err("Value must be non-negative".to_string());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, or C".to_string());
    }
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
    let variance: f64 = if count > 0.0 {
        records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count
    } else {
        0.0
    };
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,A").unwrap();
        writeln!(temp_file, "2,Test2,20.0,B").unwrap();
        
        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
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

pub fn process_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value < 0.0 {
            return Err("Negative value found in data".into());
        }
        
        if record.name.trim().is_empty() {
            return Err("Empty name field detected".into());
        }
        
        records.push(record);
    }
    
    if records.is_empty() {
        return Err("No valid records found in CSV file".into());
    }
    
    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let average = sum / count;
    
    let max_value = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);
    
    let min_value = records.iter()
        .map(|r| r.value)
        .fold(f64::INFINITY, f64::min);
    
    (average, min_value, max_value)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_process_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        
        let result = process_csv_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, category: "A".to_string() },
        ];
        
        let (avg, min, max) = calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert_eq!(min, 10.0);
        assert_eq!(max, 30.0);
    }
    
    #[test]
    fn test_category_filter() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, category: "A".to_string() },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, category: "B".to_string() },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, category: "A".to_string() },
        ];
        
        let filtered = filter_by_category(records, "A");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "A"));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
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

    pub fn process_csv(&self, filter_column: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut filtered_data = Vec::new();
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                filtered_data.push(line.split(',').map(|s| s.to_string()).collect());
                continue;
            }
            
            let columns: Vec<&str> = line.split(',').collect();
            
            if columns.get(filter_column).map_or(false, |&val| val == filter_value) {
                filtered_data.push(columns.iter().map(|&s| s.to_string()).collect());
            }
        }
        
        Ok(filtered_data)
    }
    
    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let count = reader.lines().count();
        Ok(count.saturating_sub(1))
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
        writeln!(temp_file, "id,name,status").unwrap();
        writeln!(temp_file, "1,Alice,active").unwrap();
        writeln!(temp_file, "2,Bob,inactive").unwrap();
        writeln!(temp_file, "3,Charlie,active").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        
        let result = processor.process_csv(2, "active").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[1][0], "1");
        assert_eq!(result[2][0], "3");
        
        let count = processor.count_records().unwrap();
        assert_eq!(count, 3);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
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
        if threshold <= 0.0 || threshold >= 100.0 {
            return Err(ValidationError {
                message: format!("Threshold must be between 0 and 100, got {}", threshold),
            });
        }
        Ok(DataProcessor { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Vec<f64> {
        values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .map(|&v| v * 2.0)
            .collect()
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        let count = values.len() as f64;
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(25.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(150.0);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(10.0).unwrap();
        let values = vec![5.0, 15.0, 25.0, 35.0];
        let result = processor.process_values(&values);
        assert_eq!(result, vec![30.0, 50.0, 70.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Processing timeout")]
    Timeout,
    #[error("Serialization error")]
    SerializationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_value(&mut self, key: &str, value: f64) -> Result<(), DataError> {
        if !value.is_finite() {
            return Err(DataError::InvalidInput(
                "Value must be finite number".to_string(),
            ));
        }
        self.values.insert(key.to_string(), value);
        Ok(())
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.values.is_empty() {
            return Err(DataError::InvalidInput(
                "Record must contain at least one value".to_string(),
            ));
        }
        if self.timestamp < 0 {
            return Err(DataError::InvalidInput(
                "Timestamp cannot be negative".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct DataProcessor {
    max_records: usize,
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new(max_records: usize) -> Self {
        Self {
            max_records,
            records: Vec::with_capacity(max_records),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;

        if self.records.len() >= self.max_records {
            return Err(DataError::InvalidInput(
                "Maximum record limit reached".to_string(),
            ));
        }

        self.records.push(record);
        Ok(())
    }

    pub fn process_records(&mut self) -> Result<Vec<ProcessedData>, DataError> {
        let mut results = Vec::new();

        for record in &self.records {
            let processed = self.process_single_record(record)?;
            results.push(processed);
        }

        Ok(results)
    }

    fn process_single_record(&self, record: &DataRecord) -> Result<ProcessedData, DataError> {
        let total: f64 = record.values.values().sum();
        let count = record.values.len();
        let average = if count > 0 { total / count as f64 } else { 0.0 };

        let max_value = record
            .values
            .values()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(0.0);

        let min_value = record
            .values
            .values()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(0.0);

        Ok(ProcessedData {
            record_id: record.id,
            timestamp: record.timestamp,
            value_count: count,
            average_value: average,
            max_value,
            min_value,
            has_tags: !record.tags.is_empty(),
        })
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[derive(Debug, Serialize)]
pub struct ProcessedData {
    pub record_id: u64,
    pub timestamp: i64,
    pub value_count: usize,
    pub average_value: f64,
    pub max_value: f64,
    pub min_value: f64,
    pub has_tags: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let mut record = DataRecord::new(1, 1234567890);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert!(record.values.is_empty());
        assert!(record.tags.is_empty());

        record.add_tag("test");
        assert_eq!(record.tags.len(), 1);
        assert!(record.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_value_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        assert!(record.add_value("temp", 25.5).is_ok());
        assert!(record.add_value("invalid", f64::INFINITY).is_err());
    }

    #[test]
    fn test_processor_limits() {
        let mut processor = DataProcessor::new(2);
        let record1 = DataRecord::new(1, 1234567890);
        let record2 = DataRecord::new(2, 1234567891);
        let record3 = DataRecord::new(3, 1234567892);

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        assert!(processor.add_record(record3).is_err());
    }
}