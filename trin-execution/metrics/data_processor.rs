use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Self {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

pub struct DataProcessor;

impl DataProcessor {
    pub fn load_from_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        let mut records = Vec::new();

        for result in reader.deserialize() {
            let record: Record = result?;
            record.validate()?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn save_to_csv<P: AsRef<Path>>(records: &[Record], path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = Writer::from_path(path)?;

        for record in records {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
        if records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let mean = sum / count;

        let variance: f64 = records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_active(records: &[Record]) -> Vec<&Record> {
        records.iter().filter(|r| r.active).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_name = Record::new(2, "".to_string(), 50.0, false);
        assert!(invalid_name.validate().is_err());

        let invalid_value = Record::new(3, "Test".to_string(), -10.0, true);
        assert!(invalid_value.validate().is_err());
    }

    #[test]
    fn test_csv_roundtrip() -> Result<(), Box<dyn Error>> {
        let records = vec![
            Record::new(1, "Alpha".to_string(), 10.5, true),
            Record::new(2, "Beta".to_string(), 20.0, false),
            Record::new(3, "Gamma".to_string(), 30.75, true),
        ];

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        DataProcessor::save_to_csv(&records, path)?;
        let loaded = DataProcessor::load_from_csv(path)?;

        assert_eq!(records.len(), loaded.len());
        for (original, loaded) in records.iter().zip(loaded.iter()) {
            assert_eq!(original.id, loaded.id);
            assert_eq!(original.name, loaded.name);
            assert!((original.value - loaded.value).abs() < f64::EPSILON);
            assert_eq!(original.active, loaded.active);
        }

        Ok(())
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 20.0, true),
            Record::new(3, "C".to_string(), 30.0, true),
        ];

        let (mean, variance, std_dev) = DataProcessor::calculate_statistics(&records);
        assert!((mean - 20.0).abs() < f64::EPSILON);
        assert!((variance - 66.66666666666667).abs() < f64::EPSILON);
        assert!((std_dev - 8.16496580927726).abs() < f64::EPSILON);
    }

    #[test]
    fn test_filter_active() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 20.0, false),
            Record::new(3, "C".to_string(), 30.0, true),
        ];

        let active = DataProcessor::filter_active(&records);
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|r| r.active));
    }
}
use std::collections::HashMap;

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

        self.validate_data(data)?;
        
        let processed_data = self.transform_data(data);
        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<(), String> {
        for value in data {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }
        Ok(())
    }

    fn transform_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = self.calculate_mean(data);
        data.iter()
            .map(|&x| (x - mean).abs())
            .collect()
    }

    fn calculate_mean(&self, data: &[f64]) -> f64 {
        let sum: f64 = data.iter().sum();
        sum / data.len() as f64
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
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
        let test_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_dataset("test", &test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), test_data.len());
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("empty", &[]);
        assert!(result.is_err());
    }
}use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let mut result = Vec::with_capacity(data.len());
        
        for &value in data {
            if value.is_nan() || value.is_infinite() {
                return Err(format!("Invalid numeric value detected: {}", value));
            }
            if value < 0.0 {
                return Err("Negative values not allowed in this context".to_string());
            }
            result.push(value);
        }
        
        Ok(result)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let sum: f64 = data.iter().sum();
        if sum == 0.0 {
            return vec![0.0; data.len()];
        }
        
        data.iter()
            .map(|&x| x / sum)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| (x * 100.0).ln_1p())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
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
        
        let sum: f64 = normalized.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        
        let result1 = processor.process_dataset("test", &data);
        let result2 = processor.process_dataset("test", &data);
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
        
        let (unique_keys, total_values) = processor.cache_stats();
        assert_eq!(unique_keys, 1);
        assert_eq!(total_values, 3);
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
    pub timestamp: String,
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

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord {
                id,
                value,
                category: parts[2].to_string(),
                timestamp: parts[3].to_string(),
            };

            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_records_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
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
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,electronics,2024-01-01").unwrap();
        writeln!(temp_file, "2,15.3,furniture,2024-01-02").unwrap();
        writeln!(temp_file, "3,8.7,electronics,2024-01-03").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_records_count(), 3);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 11.5).abs() < 0.001);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::DuplicateTag => write!(f, "Tags must be unique"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let mut seen_tags = HashMap::new();
        for tag in &self.tags {
            if seen_tags.insert(tag, true).is_some() {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }
    
    pub fn normalize(&mut self) {
        self.name = self.name.trim().to_string();
        self.tags.sort();
        self.tags.dedup();
    }
    
    pub fn transform_value(&mut self, multiplier: f64) -> Result<(), Box<dyn Error>> {
        if multiplier <= 0.0 {
            return Err("Multiplier must be positive".into());
        }
        
        self.value *= multiplier;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut valid_records = Vec::new();
    
    for record in records {
        if let Err(e) = record.validate() {
            eprintln!("Skipping invalid record {}: {}", record.id, e);
            continue;
        }
        
        record.normalize();
        
        if let Err(e) = record.transform_value(1.5) {
            eprintln!("Failed to transform record {}: {}", record.id, e);
            continue;
        }
        
        valid_records.push(record.clone());
    }
    
    if valid_records.is_empty() {
        return Err("No valid records found after processing".into());
    }
    
    Ok(valid_records)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 10.0,
            tags: vec![],
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_normalize() {
        let mut record = DataRecord {
            id: 1,
            name: "  Test  ".to_string(),
            value: 10.0,
            tags: vec!["b".to_string(), "a".to_string(), "b".to_string()],
        };
        
        record.normalize();
        
        assert_eq!(record.name, "Test");
        assert_eq!(record.tags, vec!["a", "b"]);
    }
}