use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
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
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(_) => continue,
            }
        }

        Ok(count)
    }

    pub fn total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value() / self.records.len() as f64)
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn max_value_record(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_data_record() {
        let record = DataRecord::new(1, -5.0, "test".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_calculate_adjusted_value() {
        let record = DataRecord::new(1, 10.0, "test".to_string()).unwrap();
        assert_eq!(record.calculate_adjusted_value(2.5), 25.0);
    }

    #[test]
    fn test_load_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,invalid,category_c").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.total_value(), 30.8);
    }

    #[test]
    fn test_average_value() {
        let mut processor = DataProcessor::new();
        assert!(processor.average_value().is_none());

        processor
            .records
            .push(DataRecord::new(1, 10.0, "test".to_string()).unwrap());
        processor
            .records
            .push(DataRecord::new(2, 20.0, "test".to_string()).unwrap());
        assert_eq!(processor.average_value(), Some(15.0));
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor
            .records
            .push(DataRecord::new(1, 10.0, "cat_a".to_string()).unwrap());
        processor
            .records
            .push(DataRecord::new(2, 20.0, "cat_b".to_string()).unwrap());
        processor
            .records
            .push(DataRecord::new(3, 30.0, "cat_a".to_string()).unwrap());

        let filtered = processor.filter_by_category("cat_a");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
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

    pub fn add_value(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn add_tag(&mut self, tag: &str) {
        self.tags.push(tag.to_string());
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".to_string()));
        }

        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(DataError::ValidationFailed(
                "Record must contain at least one value".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed(
                    "Value key cannot be empty".to_string(),
                ));
            }
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::ValidationFailed(format!(
                    "Invalid value for key '{}'",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&mut self) {
        let sum: f64 = self.values.values().sum();
        if sum != 0.0 {
            for value in self.values.values_mut() {
                *value /= sum;
            }
        }
    }

    pub fn contains_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    pub fn get_average_value(&self) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.values.values().sum::<f64>() / self.values.len() as f64
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.normalize_values();
        processed.push(processed_record);
    }

    Ok(processed)
}

pub fn filter_records_by_tag(records: &[DataRecord], tag: &str) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.contains_tag(tag))
        .cloned()
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();

    if records.is_empty() {
        return stats;
    }

    let count = records.len() as f64;
    let mut value_sums: HashMap<String, f64> = HashMap::new();
    let mut value_counts: HashMap<String, usize> = HashMap::new();

    for record in records {
        for (key, value) in &record.values {
            *value_sums.entry(key.clone()).or_insert(0.0) += value;
            *value_counts.entry(key.clone()).or_insert(0) += 1;
        }
    }

    for (key, sum) in value_sums {
        if let Some(count) = value_counts.get(&key) {
            stats.insert(format!("{}_average", key), sum / *count as f64);
        }
    }

    stats.insert("total_records".to_string(), count);
    stats.insert(
        "average_values_per_record".to_string(),
        value_counts.values().sum::<usize>() as f64 / count,
    );

    stats
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u64,
    category: String,
    value: f64,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, category: String, value: f64) -> Self {
        Self {
            id,
            category,
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid ID: must be greater than 0".into());
        }
        
        if self.category.is_empty() {
            return Err("Category cannot be empty".into());
        }

        if !self.value.is_finite() {
            return Err("Value must be a finite number".into());
        }

        Ok(())
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
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
    let mut processed = Vec::new();
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = DataRecord::new(
            record.id,
            record.category.clone(),
            record.value,
        );
        
        for (key, value) in &record.metadata {
            processed_record.add_metadata(key.clone(), value.clone());
        }
        
        processed_record.transform_value(1.5)?;
        processed.push(processed_record);
    }
    
    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let avg = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - avg).powi(2))
        .sum::<f64>() / count;
    
    stats.insert("total_records".to_string(), count);
    stats.insert("sum".to_string(), sum);
    stats.insert("average".to_string(), avg);
    stats.insert("variance".to_string(), variance);
    
    stats
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let value: f64 = line.trim().parse()?;
            self.data.push(value);
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

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        Some(variance.sqrt())
    }

    pub fn filter_outliers(&self, threshold: f64) -> Vec<f64> {
        let mean = match self.calculate_mean() {
            Some(m) => m,
            None => return Vec::new(),
        };
        let std_dev = match self.calculate_standard_deviation() {
            Some(s) => s,
            None => return Vec::new(),
        };

        self.data
            .iter()
            .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
            .cloned()
            .collect()
    }

    pub fn get_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let std_dev = self.calculate_standard_deviation().unwrap_or(0.0);
        let min = self.data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        format!(
            "Statistics Summary:\n  Count: {}\n  Mean: {:.4}\n  Std Dev: {:.4}\n  Min: {:.4}\n  Max: {:.4}",
            self.data.len(),
            mean,
            std_dev,
            min,
            max
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
        writeln!(temp_file, "10.5\n15.2\n12.8\n14.1\n13.7").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.data.len(), 5);
        assert!(processor.calculate_mean().unwrap() - 13.26 < 0.01);
        assert!(processor.calculate_standard_deviation().unwrap() - 1.67 < 0.01);
        
        let filtered = processor.filter_outliers(2.0);
        assert_eq!(filtered.len(), 5);
    }
}