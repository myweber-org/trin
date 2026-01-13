use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_data(input_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
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
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
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
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = Reader::from_reader(file);

        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value > 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn get_active_records(&self) -> Vec<&Record> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,20.0,false").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 2);
        assert_eq!(processor.calculate_total(), 30.5);
        assert_eq!(processor.get_active_records().len(), 1);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidId => write!(f, "ID must be greater than zero"),
            ProcessingError::EmptyName => write!(f, "Name cannot be empty"),
            ProcessingError::NegativeValue => write!(f, "Value cannot be negative"),
            ProcessingError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, ProcessingError> {
        let record = DataRecord { id, name, value, tags };
        record.validate()?;
        Ok(record)
    }

    fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ProcessingError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ProcessingError::NegativeValue);
        }
        
        let mut seen_tags = HashMap::new();
        for tag in &self.tags {
            if seen_tags.contains_key(tag) {
                return Err(ProcessingError::DuplicateTag);
            }
            seen_tags.insert(tag.clone(), true);
        }
        
        Ok(())
    }

    pub fn transform(&self, multiplier: f64) -> Self {
        DataRecord {
            id: self.id,
            name: self.name.clone(),
            value: self.value * multiplier,
            tags: self.tags.clone(),
        }
    }

    pub fn add_tag(&mut self, tag: String) -> Result<(), ProcessingError> {
        if self.tags.contains(&tag) {
            return Err(ProcessingError::DuplicateTag);
        }
        self.tags.push(tag);
        Ok(())
    }

    pub fn calculate_score(&self) -> f64 {
        let base_score = self.value * 100.0;
        let tag_bonus = self.tags.len() as f64 * 10.0;
        base_score + tag_bonus
    }
}

pub fn process_records(records: Vec<DataRecord>, multiplier: f64) -> Vec<DataRecord> {
    records
        .into_iter()
        .map(|record| record.transform(multiplier))
        .collect()
}

pub fn filter_records_by_threshold(records: Vec<DataRecord>, threshold: f64) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.value >= threshold)
        .collect()
}

pub fn calculate_average_value(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            42.5,
            vec!["tag1".to_string(), "tag2".to_string()]
        );
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "Test".to_string(),
            10.0,
            vec![]
        );
        assert!(matches!(record, Err(ProcessingError::InvalidId)));
    }

    #[test]
    fn test_duplicate_tags() {
        let record = DataRecord::new(
            1,
            "Test".to_string(),
            10.0,
            vec!["tag1".to_string(), "tag1".to_string()]
        );
        assert!(matches!(record, Err(ProcessingError::DuplicateTag)));
    }

    #[test]
    fn test_transform() {
        let record = DataRecord::new(1, "Test".to_string(), 10.0, vec![]).unwrap();
        let transformed = record.transform(2.0);
        assert_eq!(transformed.value, 20.0);
    }

    #[test]
    fn test_calculate_score() {
        let record = DataRecord::new(
            1,
            "Test".to_string(),
            5.0,
            vec!["tag1".to_string(), "tag2".to_string()]
        ).unwrap();
        let score = record.calculate_score();
        assert_eq!(score, 5.0 * 100.0 + 2.0 * 10.0);
    }
}