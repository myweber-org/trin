
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
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

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<usize, String> {
        if records.is_empty() {
            return Err("No valid records found".to_string());
        }

        let expected_len = records[0].len();
        for (i, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!(
                    "Record {} has {} fields, expected {}",
                    i + 1,
                    record.len(),
                    expected_len
                ));
            }
        }

        Ok(records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_validation() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let result = processor.validate_records(&records);
        
        assert_eq!(result, Ok(2));
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
    pub tags: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    ValueOutOfRange(f64),
    #[error("Duplicate record ID: {0}")]
    DuplicateId(u64),
}

pub struct DataProcessor {
    validation_enabled: bool,
    max_value_limit: Option<f64>,
    seen_ids: std::collections::HashSet<u64>,
}

impl DataProcessor {
    pub fn new(validation_enabled: bool, max_value_limit: Option<f64>) -> Self {
        Self {
            validation_enabled,
            max_value_limit,
            seen_ids: std::collections::HashSet::new(),
        }
    }

    pub fn process_record(&mut self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        if self.validation_enabled {
            self.validate_record(record)?;
        }
        
        let transformed = self.transform_record(record);
        Ok(transformed)
    }

    fn validate_record(&mut self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        if record.values.is_empty() {
            return Err(ProcessingError::MissingField("values".to_string()));
        }

        if self.seen_ids.contains(&record.id) {
            return Err(ProcessingError::DuplicateId(record.id));
        }

        if let Some(limit) = self.max_value_limit {
            for &value in record.values.values() {
                if value > limit {
                    return Err(ProcessingError::ValueOutOfRange(value));
                }
            }
        }

        self.seen_ids.insert(record.id);
        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> DataRecord {
        let mut transformed_values = HashMap::new();
        
        for (key, value) in &record.values {
            let transformed_key = key.to_lowercase().replace(' ', "_");
            let transformed_value = if *value < 0.0 {
                0.0
            } else {
                *value
            };
            transformed_values.insert(transformed_key, transformed_value);
        }

        let mut transformed_tags = record.tags.clone();
        transformed_tags.sort();
        transformed_tags.dedup();

        DataRecord {
            id: record.id,
            timestamp: record.timestamp,
            values: transformed_values,
            tags: transformed_tags,
        }
    }

    pub fn reset_processor(&mut self) {
        self.seen_ids.clear();
    }

    pub fn get_processed_count(&self) -> usize {
        self.seen_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let mut processor = DataProcessor::new(true, Some(100.0));
        
        let mut values = HashMap::new();
        values.insert("Temperature".to_string(), 25.5);
        values.insert("Pressure".to_string(), 101.3);
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values,
            tags: vec!["sensor".to_string(), "room1".to_string()],
        };

        let result = processor.process_record(&record);
        assert!(result.is_ok());
        assert_eq!(processor.get_processed_count(), 1);
    }

    #[test]
    fn test_invalid_timestamp() {
        let mut processor = DataProcessor::new(true, None);
        
        let record = DataRecord {
            id: 1,
            timestamp: -1,
            values: HashMap::from([("test".to_string(), 1.0)]),
            tags: vec![],
        };

        let result = processor.process_record(&record);
        assert!(matches!(result, Err(ProcessingError::InvalidTimestamp(-1))));
    }

    #[test]
    fn test_value_limit_exceeded() {
        let mut processor = DataProcessor::new(true, Some(50.0));
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: HashMap::from([("reading".to_string(), 75.0)]),
            tags: vec![],
        };

        let result = processor.process_record(&record);
        assert!(matches!(result, Err(ProcessingError::ValueOutOfRange(75.0))));
    }
}
use std::error::Error;
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
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();
            let valid = parts[3].parse::<bool>().unwrap_or(false);
            
            let record = DataRecord {
                id,
                value,
                category,
                valid,
            };
            
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.valid)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.filter_valid();
        if valid_records.is_empty() {
            return None;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
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
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,category_a,true").unwrap();
        writeln!(temp_file, "2,20.3,category_b,false").unwrap();
        writeln!(temp_file, "3,15.7,category_a,true").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 13.1).abs() < 0.001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.get("category_a").unwrap().len(), 2);
        assert_eq!(groups.get("category_b").unwrap().len(), 1);
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
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "A".to_string(),
        };
        assert!(validate_record(&valid_record).is_ok());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "D".to_string(),
        };
        assert!(validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_calculate_statistics() {
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

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub struct DataProcessor {
    records: Vec<Record>,
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

            let active = match parts[3].to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                _ => false,
            };

            let record = Record::new(id, name, value, active);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value > threshold)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn count_active(&self) -> usize {
        self.records.iter().filter(|record| record.active).count()
    }

    pub fn get_records(&self) -> &[Record] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Alice,100.5,true").unwrap();
        writeln!(temp_file, "2,Bob,50.0,false").unwrap();
        writeln!(temp_file, "3,Charlie,75.3,true").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.get_records().len(), 3);
        
        let filtered = processor.filter_by_value(60.0);
        assert_eq!(filtered.len(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 75.26666666666667).abs() < 0.0001);
        
        assert_eq!(processor.count_active(), 2);
    }
}