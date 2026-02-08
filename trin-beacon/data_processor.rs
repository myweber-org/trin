
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub category: String,
    pub count: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        self.records.push(record.clone());
        self.update_category_stats(&record);
        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string(),
            ));
        }

        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError(
                "Record value cannot be negative".to_string(),
            ));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record category cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                category: record.category.clone(),
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.count += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.count as f64;
    }

    pub fn get_records_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn calculate_overall_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) -> Result<(), ProcessingError>
    where
        F: Fn(f64) -> Result<f64, ProcessingError>,
    {
        for record in &mut self.records {
            match transform_fn(record.value) {
                Ok(new_value) => record.value = new_value,
                Err(e) => return Err(e),
            }
        }
        self.recalculate_stats();
        Ok(())
    }

    fn recalculate_stats(&mut self) {
        self.category_stats.clear();
        for record in &self.records {
            self.update_category_stats(record);
        }
    }

    pub fn get_total_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.category_stats.clear();
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
    #[error("Duplicate record ID: {0}")]
    DuplicateId(u64),
}

pub struct DataProcessor {
    cache: HashMap<u64, DataRecord>,
    validation_rules: ValidationRules,
}

#[derive(Clone)]
pub struct ValidationRules {
    pub max_values: usize,
    pub allowed_tags: Vec<String>,
    pub timestamp_range: (i64, i64),
}

impl DataProcessor {
    pub fn new(rules: ValidationRules) -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn process_record(&mut self, record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;
        
        if self.cache.contains_key(&record.id) {
            return Err(ProcessingError::DuplicateId(record.id));
        }

        let processed = self.transform_record(record.clone());
        self.cache.insert(processed.id, processed.clone());
        
        Ok(processed)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.validation_rules.max_values {
            return Err(ProcessingError::OutOfRange(
                format!("Too many values: {}", record.values.len())
            ));
        }

        if record.timestamp < self.validation_rules.timestamp_range.0 
            || record.timestamp > self.validation_rules.timestamp_range.1 {
            return Err(ProcessingError::OutOfRange(
                format!("Timestamp out of range: {}", record.timestamp)
            ));
        }

        for tag in &record.tags {
            if !self.validation_rules.allowed_tags.contains(tag) {
                return Err(ProcessingError::InvalidFormat);
            }
        }

        Ok(())
    }

    fn transform_record(&self, mut record: DataRecord) -> DataRecord {
        let normalized_values: HashMap<String, f64> = record.values
            .iter()
            .map(|(k, v)| (k.to_lowercase(), *v))
            .collect();
        
        record.values = normalized_values;
        record.tags.sort();
        record.tags.dedup();
        
        record
    }

    pub fn get_record(&self, id: u64) -> Option<&DataRecord> {
        self.cache.get(&id)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rules() -> ValidationRules {
        ValidationRules {
            max_values: 10,
            allowed_tags: vec!["production".to_string(), "test".to_string()],
            timestamp_range: (0, 1000000000),
        }
    }

    #[test]
    fn test_valid_record_processing() {
        let mut processor = DataProcessor::new(create_test_rules());
        let mut values = HashMap::new();
        values.insert("temperature".to_string(), 23.5);
        
        let record = DataRecord {
            id: 1,
            timestamp: 123456789,
            values,
            tags: vec!["production".to_string()],
        };

        let result = processor.process_record(record);
        assert!(result.is_ok());
        assert_eq!(processor.cache_size(), 1);
    }

    #[test]
    fn test_duplicate_id() {
        let mut processor = DataProcessor::new(create_test_rules());
        let record1 = DataRecord {
            id: 1,
            timestamp: 100,
            values: HashMap::new(),
            tags: vec![],
        };
        
        let record2 = DataRecord {
            id: 1,
            timestamp: 200,
            values: HashMap::new(),
            tags: vec![],
        };

        let _ = processor.process_record(record1);
        let result = processor.process_record(record2);
        
        assert!(matches!(result, Err(ProcessingError::DuplicateId(1))));
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub fn process_csv_file(input_path: &str, output_path: &str) -> Result<usize, Box<dyn Error>> {
    let path = Path::new(input_path);
    if !path.exists() {
        return Err("Input file does not exist".into());
    }

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(input_path)?;

    let mut valid_records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        if record.is_valid() {
            valid_records.push(record);
        }
    }

    if valid_records.is_empty() {
        return Err("No valid records found".into());
    }

    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(output_path)?;

    let mut processed_count = 0;
    for record in valid_records {
        writer.serialize(&record)?;
        processed_count += 1;
    }

    writer.flush()?;
    Ok(processed_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_valid_record() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = Record {
            id: 2,
            name: "".to_string(),
            value: -50.0,
            category: "".to_string(),
        };
        assert!(!record.is_valid());
    }
}
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
    
    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;
        
        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
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
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
    }
    
    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["test".to_string(), "123".to_string()];
        let invalid_record = vec!["".to_string(), "data".to_string()];
        
        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }
    
    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(',', false);
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "30.0".to_string()],
            vec!["invalid".to_string(), "40.0".to_string()],
        ];
        
        let avg = processor.calculate_statistics(&records, 0);
        assert_eq!(avg, Some(13.0));
        
        let invalid_avg = processor.calculate_statistics(&records, 2);
        assert_eq!(invalid_avg, None);
    }
}