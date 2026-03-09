
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
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,test1,10.5").unwrap();
        writeln!(temp_file, "2,test2,20.3").unwrap();
        writeln!(temp_file, "3,test3,15.7").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], vec!["1", "test1", "10.5"]);

        let stats = processor.calculate_statistics(&data, 2);
        assert!(stats.is_some());
        
        let (mean, variance, std_dev) = stats.unwrap();
        assert!((mean - 15.5).abs() < 0.1);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
    }

    #[test]
    fn test_invalid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,,30.5").unwrap();

        let processor = DataProcessor::new(',', false);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_err());
    }
}
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

    pub fn process_numeric_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data array provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(values)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        for &value in data {
            if !value.is_finite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }
        Ok(data.to_vec())
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).ln_1p())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_entries = self.cache.len();
        let total_values = self.cache.values()
            .map(|v| v.len())
            .sum();
        (total_entries, total_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_validation() {
        let processor = DataProcessor::new();
        let valid_data = vec![1.0, 2.0, 3.0];
        let invalid_data = vec![1.0, f64::NAN, 3.0];

        assert!(processor.validate_data(&valid_data).is_ok());
        assert!(processor.validate_data(&invalid_data).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        let normalized = processor.normalize_data(&data);

        assert_eq!(normalized.len(), 3);
        let sum: f64 = normalized.iter().sum();
        assert!(sum.abs() < 1e-10);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];

        let result1 = processor.process_numeric_data("test", &data);
        let result2 = processor.process_numeric_data("test", &data);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());

        let stats = processor.get_cache_stats();
        assert_eq!(stats.0, 1);
        assert_eq!(stats.1, 3);

        processor.clear_cache();
        let stats_after_clear = processor.get_cache_stats();
        assert_eq!(stats_after_clear.0, 0);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    (sum, average, count)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
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
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, ProcessingError> {
        if name.is_empty() {
            return Err(ProcessingError::InvalidData("Name cannot be empty".to_string()));
        }
        if value < 0.0 {
            return Err(ProcessingError::InvalidData("Value must be non-negative".to_string()));
        }
        
        Ok(Self { id, name, value, tags })
    }
    
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.name.len() > 100 {
            return Err(ProcessingError::ValidationError("Name too long".to_string()));
        }
        if self.value > 1000000.0 {
            return Err(ProcessingError::ValidationError("Value exceeds maximum".to_string()));
        }
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::TransformationFailed("Multiplier must be positive".to_string()));
        }
        
        self.value *= multiplier;
        self.tags.push(format!("transformed_{}", multiplier));
        Ok(())
    }
    
    pub fn calculate_score(&self) -> f64 {
        let base_score = self.value * 0.8;
        let tag_bonus = self.tags.len() as f64 * 0.2;
        base_score + tag_bonus
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    transformations_applied: u32,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            transformations_applied: 0,
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        record.validate()?;
        
        if self.records.contains_key(&record.id) {
            return Err(ProcessingError::InvalidData(format!("Duplicate ID: {}", record.id)));
        }
        
        self.records.insert(record.id, record);
        Ok(())
    }
    
    pub fn process_all(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        for (_, record) in self.records.iter_mut() {
            record.transform(multiplier)?;
            self.transformations_applied += 1;
        }
        Ok(())
    }
    
    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        let count = self.records.len() as f64;
        let total_value: f64 = self.records.values().map(|r| r.value).sum();
        let avg_score: f64 = self.records.values().map(|r| r.calculate_score()).sum::<f64>() / count;
        
        stats.insert("record_count".to_string(), count);
        stats.insert("total_value".to_string(), total_value);
        stats.insert("average_score".to_string(), avg_score);
        stats.insert("transformations_applied".to_string(), self.transformations_applied as f64);
        
        stats
    }
    
    pub fn find_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.tags.iter().any(|t| t == tag))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, "Test".to_string(), 42.5, vec!["tag1".to_string()]);
        assert!(record.is_ok());
    }
    
    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, "".to_string(), 10.0, vec![]);
        assert!(record.is_err());
    }
    
    #[test]
    fn test_transformation() {
        let mut record = DataRecord::new(3, "Sample".to_string(), 10.0, vec![]).unwrap();
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 20.0);
    }
    
    #[test]
    fn test_processor_operations() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(1, "Data1".to_string(), 100.0, vec!["test".to_string()]).unwrap();
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }
}