
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
            return Err("Empty data set provided".to_string());
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

    fn validate_data(&self, values: &[f64]) -> Result<Vec<f64>, String> {
        let mut valid_values = Vec::new();
        
        for &value in values {
            if value.is_finite() {
                valid_values.push(value);
            } else {
                return Err(format!("Invalid numeric value detected: {}", value));
            }
        }

        if valid_values.len() < 2 {
            return Err("Insufficient valid data points".to_string());
        }

        Ok(valid_values)
    }

    fn normalize_data(&self, values: &[f64]) -> Vec<f64> {
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return vec![0.0; values.len()];
        }

        values.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn apply_transformations(&self, values: &[f64]) -> Vec<f64> {
        values.iter()
            .map(|&x| x.powi(2).ln_1p())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_items = self.cache.len();
        let total_values = self.cache.values()
            .map(|v| v.len())
            .sum();
        
        (total_items, total_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let test_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_numeric_data("test", &test_data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), test_data.len());
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let invalid_data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_numeric_data("invalid", &invalid_data);
        assert!(result.is_err());
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
            let valid = value >= 0.0 && value <= 1000.0;
            
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

    pub fn count_valid(&self) -> usize {
        self.records.iter().filter(|r| r.valid).count()
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
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,CategoryA").unwrap();
        writeln!(temp_file, "2,200.0,CategoryB").unwrap();
        writeln!(temp_file, "3,150.75,CategoryA").unwrap();
        writeln!(temp_file, "4,-10.0,CategoryC").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
        assert_eq!(processor.count_records(), 4);
        assert_eq!(processor.count_valid(), 3);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 150.41666666666666).abs() < 0.0001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("CategoryA").unwrap().len(), 2);
        assert_eq!(groups.get("CategoryB").unwrap().len(), 1);
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

    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 3 {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
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
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.0,beta").unwrap();
        writeln!(temp_file, "3,-5.0,gamma").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "a"));
        processor.records.push(DataRecord::new(2, 20.0, "b"));
        processor.records.push(DataRecord::new(3, -5.0, "c"));

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(15.0));
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

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_rejects_invalid() {
        let processor = DataProcessor::new();
        let data = [1.0, f64::NAN, 3.0];
        assert!(processor.validate_data(&data).is_err());
    }

    #[test]
    fn test_normalization_produces_zero_mean() {
        let processor = DataProcessor::new();
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = processor.normalize_data(&data);
        let mean = normalized.iter().sum::<f64>() / normalized.len() as f64;
        assert!(mean.abs() < 1e-10);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = [1.5, 2.5, 3.5];
        
        let result1 = processor.process_dataset("test", &data).unwrap();
        let result2 = processor.process_dataset("test", &data).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(processor.cache_stats(), (1, 3));
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
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_data(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if record.name.is_empty() {
                errors.push(format!("Record {}: Name cannot be empty", index));
            }

            if record.value < 0.0 {
                errors.push(format!("Record {}: Value cannot be negative", index));
            }

            if !["A", "B", "C"].contains(&record.category.as_str()) {
                errors.push(format!("Record {}: Invalid category", index));
            }
        }

        errors
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,A").unwrap();
        writeln!(temp_file, "2,Test2,20.0,B").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 2);
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats.0, 15.25);
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 1);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Values cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count as f64;
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        let min = self.values.iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some(DataStatistics {
            count,
            mean,
            std_dev,
            min,
            max,
            sum,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStatistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

pub fn transform_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records.into_iter()
        .filter(|record| record.validate().is_ok())
        .map(|mut record| {
            record.timestamp += 3600;
            record
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut valid_record = DataRecord::new(1, 1625097600);
        valid_record.add_value(42.0);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1625097600);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value(10.0);
        record.add_value(20.0);
        record.add_value(30.0);

        let stats = record.calculate_statistics().unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.sum, 60.0);
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, 1625097600);
        record.add_value(42.0);
        
        let transformed = transform_records(vec![record.clone()]);
        assert_eq!(transformed[0].timestamp, record.timestamp + 3600);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: i64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    TimestampOutOfRange,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::TimestampOutOfRange => write!(f, "Timestamp must be non-negative"),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        if timestamp < 0 {
            return Err(DataError::TimestampOutOfRange);
        }

        Ok(Self {
            id,
            value,
            timestamp,
        })
    }

    pub fn transform_value(&mut self, multiplier: f64) -> Result<(), DataError> {
        let new_value = self.value * multiplier;
        if new_value < 0.0 || new_value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        self.value = new_value;
        Ok(())
    }

    pub fn normalize(&self) -> f64 {
        self.value / 1000.0
    }

    pub fn is_anomaly(&self, threshold: f64) -> bool {
        self.normalize() > threshold
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

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn process_all(&mut self, multiplier: f64) -> Result<(), Vec<DataError>> {
        let mut errors = Vec::new();

        for record in &mut self.records {
            if let Err(e) = record.transform_value(multiplier) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn find_anomalies(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.is_anomaly(threshold))
            .collect()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            let sum: f64 = self.records.iter().map(|r| r.value).sum();
            Some(sum / self.records.len() as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, 1234567890).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.timestamp, 1234567890);
    }

    #[test]
    fn test_invalid_id() {
        let result = DataRecord::new(0, 500.0, 1234567890);
        assert!(matches!(result, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 100.0, 1234567890).unwrap();
        record.transform_value(2.0).unwrap();
        assert_eq!(record.value, 200.0);
    }

    #[test]
    fn test_anomaly_detection() {
        let record = DataRecord::new(1, 800.0, 1234567890).unwrap();
        assert!(record.is_anomaly(0.7));
        assert!(!record.is_anomaly(0.9));
    }
}