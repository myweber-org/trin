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
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        let mut values = Vec::new();
        
        for record in records {
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
            .map(|v| (v - mean).powi(2))
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
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert!(processor.validate_record(&records[0]));
        
        let stats = processor.calculate_statistics(&records, 1);
        assert!(stats.is_some());
        
        if let Some((mean, _, _)) = stats {
            assert!((mean - 30.0).abs() < 0.001);
        }
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
    validation_threshold: f64,
    normalization_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, normalization_factor: f64) -> Self {
        Self {
            validation_threshold,
            normalization_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Empty values array".to_string(),
            ));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationFailed(
                    "Invalid numeric value".to_string(),
                ));
            }

            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationFailed(format!(
                    "Value {} exceeds threshold {}",
                    value, self.validation_threshold
                )));
            }
        }

        Ok(())
    }

    pub fn transform_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        for value in record.values.iter_mut() {
            *value = (*value * self.normalization_factor).round();
        }

        record.metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string(),
        );

        Ok(record)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> (Vec<DataRecord>, Vec<ProcessingError>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.transform_record(record) {
                Ok(transformed) => processed.push(transformed),
                Err(err) => errors.push(err),
            }
        }

        (processed, errors)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.clone())
            .collect();

        let sum: f64 = all_values.iter().sum();
        let mean = sum / total_values as f64;

        let variance: f64 = all_values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / total_values as f64;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 1.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 20.3, 30.7],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(10.0, 1.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 20.3, 30.7],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.2, 2.7, 3.4],
            metadata: HashMap::new(),
        };

        let transformed = processor.transform_record(record).unwrap();
        assert_eq!(transformed.values, vec![2.0, 5.0, 7.0]);
        assert!(transformed.metadata.contains_key("processed_timestamp"));
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
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
            "Data Summary:\nTotal entries: {}\nMean: {:.2}\nMedian: {:.2}\nUnique categories: {}",
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
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
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
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), Box<dyn Error>> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if record.value < 0.0 {
        return Err("Value cannot be negative".into());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Invalid category".into());
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
    }

    #[test]
    fn test_invalid_category() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,Invalid").unwrap();

        let result = process_data_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
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
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            // Skip header and empty lines
            if line_num == 0 || line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 3 {
                continue;
            }
            
            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();
            let valid = value > 0.0 && !category.is_empty();
            
            self.records.push(DataRecord {
                id,
                value,
                category,
                valid,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> f64 {
        let valid_records = self.filter_valid_records();
        if valid_records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        sum / valid_records.len() as f64
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

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let valid_values: Vec<f64> = self.filter_valid_records()
            .iter()
            .map(|r| r.value)
            .collect();
        
        if valid_values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let min = valid_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = valid_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average();
        
        (min, max, avg)
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
        
        // Create temporary CSV file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,type_a").unwrap();
        writeln!(temp_file, "2,20.3,type_b").unwrap();
        writeln!(temp_file, "3,15.7,type_a").unwrap();
        writeln!(temp_file, "4,0.0,type_c").unwrap(); // Invalid record
        
        let file_path = temp_file.path().to_str().unwrap();
        
        // Test loading
        let count = processor.load_from_csv(file_path).unwrap();
        assert_eq!(count, 4);
        
        // Test filtering
        let valid_records = processor.filter_valid_records();
        assert_eq!(valid_records.len(), 3);
        
        // Test calculations
        let avg = processor.calculate_average();
        assert!((avg - 15.5).abs() < 0.01);
        
        // Test statistics
        let (min, max, avg_stat) = processor.get_statistics();
        assert!((min - 10.5).abs() < 0.01);
        assert!((max - 20.3).abs() < 0.01);
        assert!((avg_stat - 15.5).abs() < 0.01);
        
        // Test grouping
        let groups = processor.group_by_category();
        assert_eq!(groups.get("type_a").unwrap().len(), 2);
        assert_eq!(groups.get("type_b").unwrap().len(), 1);
        assert!(!groups.contains_key("type_c"));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: String) -> Self {
        Self {
            id,
            name,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.timestamp.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
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
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let timestamp = parts[3].to_string();

            let record = DataRecord::new(id, name, value, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn filter_by_min_value(&self, min_value: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= min_value)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|record| record.id == target_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,item1,10.5,2024-01-01").unwrap();
        writeln!(temp_file, "2,item2,20.0,2024-01-02").unwrap();
        writeln!(temp_file, "3,item3,invalid,2024-01-03").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.get_records().len(), 2);
    }

    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "a".to_string(), 10.0, "t1".to_string()));
        processor.records.push(DataRecord::new(2, "b".to_string(), 20.0, "t2".to_string()));
        processor.records.push(DataRecord::new(3, "c".to_string(), 30.0, "t3".to_string()));

        let filtered = processor.filter_by_min_value(15.0);
        assert_eq!(filtered.len(), 2);

        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
    }

    #[test]
    fn test_find_by_id() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "test".to_string(), 10.0, "t1".to_string()));
        
        assert!(processor.find_by_id(1).is_some());
        assert!(processor.find_by_id(999).is_none());
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
    #[error("Value out of range: {0}")]
    OutOfRange(String),
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub metadata: Option<HashMap<String, String>>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::InvalidFormat);
        }
        
        if self.timestamp < 0 {
            return Err(ProcessingError::OutOfRange("timestamp".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(ProcessingError::MissingField("values".to_string()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(ProcessingError::InvalidFormat);
            }
            
            if !value.is_finite() {
                return Err(ProcessingError::OutOfRange(format!("value for {}", key)));
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(ProcessingError::TransformationFailed(
                "Invalid multiplier".to_string()
            ));
        }
        
        for value in self.values.values_mut() {
            *value *= multiplier;
            
            if !value.is_finite() {
                return Err(ProcessingError::TransformationFailed(
                    "Result is not finite".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }
        
        let count = self.values.len() as f64;
        let sum: f64 = self.values.values().sum();
        let avg = sum / count;
        
        let variance: f64 = self.values.values()
            .map(|v| (v - avg).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
        
        if let Some(min) = self.values.values().min_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("min".to_string(), *min);
        }
        
        if let Some(max) = self.values.values().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("max".to_string(), *max);
        }
        
        stats
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    transformation_history: Vec<String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            transformation_history: Vec::new(),
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }
    
    pub fn process_all(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        for record in &mut self.records {
            record.transform(multiplier)?;
        }
        
        self.transformation_history.push(
            format!("Applied multiplier: {}", multiplier)
        );
        
        Ok(())
    }
    
    pub fn get_summary(&self) -> HashMap<String, f64> {
        let mut summary = HashMap::new();
        
        if self.records.is_empty() {
            return summary;
        }
        
        let total_records = self.records.len() as f64;
        let mut total_values = 0.0;
        let mut all_stats = Vec::new();
        
        for record in &self.records {
            let stats = record.calculate_statistics();
            total_values += stats.get("count").unwrap_or(&0.0);
            all_stats.push(stats);
        }
        
        summary.insert("total_records".to_string(), total_records);
        summary.insert("total_values".to_string(), total_values);
        summary.insert("avg_values_per_record".to_string(), total_values / total_records);
        
        if !all_stats.is_empty() {
            let avg_avg: f64 = all_stats.iter()
                .filter_map(|s| s.get("average"))
                .sum::<f64>() / all_stats.len() as f64;
            
            summary.insert("overall_average".to_string(), avg_avg);
        }
        
        summary
    }
    
    pub fn get_transformation_history(&self) -> &[String] {
        &self.transformation_history
    }
    
    pub fn clear(&mut self) {
        self.records.clear();
        self.transformation_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("temperature".to_string(), 25.5),
                ("humidity".to_string(), 60.0),
            ]),
            metadata: None,
        };
        
        assert!(record.validate().is_ok());
        
        record.id = 0;
        assert!(record.validate().is_err());
        
        record.id = 1;
        record.values.clear();
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("value1".to_string(), 10.0),
                ("value2".to_string(), 20.0),
            ]),
            metadata: None,
        };
        
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.values.get("value1"), Some(&20.0));
        assert_eq!(record.values.get("value2"), Some(&40.0));
        
        assert!(record.transform(0.0).is_err());
        assert!(record.transform(f64::NAN).is_err());
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            timestamp: 1000,
            values: HashMap::from([("a".to_string(), 1.0)]),
            metadata: None,
        };
        
        let record2 = DataRecord {
            id: 2,
            timestamp: 2000,
            values: HashMap::from([("b".to_string(), 2.0)]),
            metadata: None,
        };
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        assert!(processor.process_all(3.0).is_ok());
        
        let summary = processor.get_summary();
        assert_eq!(summary.get("total_records"), Some(&2.0));
        
        assert_eq!(processor.get_transformation_history().len(), 1);
    }
}