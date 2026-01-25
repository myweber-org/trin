
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

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
    }

    pub fn calculate_column_average(&self, records: &[Vec<String>], column_index: usize) -> Option<f64> {
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000"]);
        
        let invalid = processor.validate_records(&records);
        assert_eq!(invalid, vec![2]);
        
        let avg_age = processor.calculate_column_average(&records, 1);
        assert_eq!(avg_age, Some(30.0));
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut values = Vec::new();

        for result in rdr.records() {
            let record = result?;
            for field in record.iter() {
                if let Ok(num) = field.parse::<f64>() {
                    values.push(num);
                }
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn variance(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.mean().unwrap();
        let sum_sq_diff: f64 = self.values.iter().map(|&x| (x - mean).powi(2)).sum();
        Some(sum_sq_diff / (self.values.len() - 1) as f64)
    }

    pub fn std_dev(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_dataset() {
        let ds = DataSet::new();
        assert_eq!(ds.count(), 0);
        assert_eq!(ds.mean(), None);
        assert_eq!(ds.variance(), None);
    }

    #[test]
    fn test_basic_statistics() {
        let mut ds = DataSet::new();
        ds.add_value(10.0);
        ds.add_value(20.0);
        ds.add_value(30.0);

        assert_eq!(ds.count(), 3);
        assert_eq!(ds.mean(), Some(20.0));
        assert_eq!(ds.variance(), Some(100.0));
        assert_eq!(ds.std_dev(), Some(10.0));
    }

    #[test]
    fn test_csv_parsing() -> Result<(), Box<dyn Error>> {
        let mut tmp_file = NamedTempFile::new()?;
        writeln!(tmp_file, "1.5,2.5,3.5\n4.5,5.5,6.5")?;

        let ds = DataSet::from_csv(tmp_file.path())?;
        assert_eq!(ds.count(), 6);
        assert!((ds.mean().unwrap() - 4.0).abs() < 1e-10);

        Ok(())
    }

    #[test]
    fn test_clear() {
        let mut ds = DataSet::new();
        ds.add_value(42.0);
        assert_eq!(ds.count(), 1);
        
        ds.clear();
        assert_eq!(ds.count(), 0);
        assert_eq!(ds.mean(), None);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
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

            let category = parts[3].to_string();

            let record = DataRecord::new(id, name, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
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

    pub fn get_statistics(&self) -> Statistics {
        let count = self.records.len();
        let avg = self.calculate_average().unwrap_or(0.0);
        let max = self.find_max_value().map(|r| r.value).unwrap_or(0.0);
        let min = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);

        Statistics {
            count,
            average: avg,
            maximum: max,
            minimum: if min == f64::INFINITY { 0.0 } else { min },
        }
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub count: usize,
    pub average: f64,
    pub maximum: f64,
    pub minimum: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Records: {}, Average: {:.2}, Max: {:.2}, Min: {:.2}",
            self.count, self.average, self.maximum, self.minimum
        )
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Data transformation failed")]
    TransformationFailed,
    #[error("Validation error: {0}")]
    ValidationError(String),
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
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationError("Timestamp cannot be negative".to_string()));
        }

        for (key, value) in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err(DataError::ValidationError(
                    format!("Invalid value for key '{}': {}", key, value)
                ));
            }
        }

        Ok(())
    }
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let mut transformed = record.clone();
        
        for (key, value) in &mut transformed.values {
            *value = transform_value(*value)?;
        }
        
        transformed.tags.sort();
        transformed.tags.dedup();
        
        processed.push(transformed);
    }
    
    processed.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    
    Ok(processed)
}

fn transform_value(value: f64) -> Result<f64, DataError> {
    if value < 0.0 {
        return Err(DataError::TransformationFailed);
    }
    
    let transformed = (value * 100.0).round() / 100.0;
    
    if transformed.is_nan() || transformed.is_infinite() {
        Err(DataError::TransformationFailed)
    } else {
        Ok(transformed)
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    for record in records {
        for (key, value) in &record.values {
            let entry = stats.entry(key.clone()).or_insert(Vec::new());
            entry.push(*value);
        }
    }
    
    let mut result = HashMap::new();
    
    for (key, values) in stats {
        if let Some(avg) = calculate_average(&values) {
            result.insert(format!("{}_avg", key), avg);
        }
        
        if let Some(max) = values.iter().copied().reduce(f64::max) {
            result.insert(format!("{}_max", key), max);
        }
        
        if let Some(min) = values.iter().copied().reduce(f64::min) {
            result.insert(format!("{}_min", key), min);
        }
    }
    
    result
}

fn calculate_average(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    
    let sum: f64 = values.iter().sum();
    Some(sum / values.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value("temperature", 25.5);
        record.add_tag("sensor");
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, -1);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let mut record1 = DataRecord::new(1, 1000);
        record1.add_value("pressure", 1013.25);
        
        let mut record2 = DataRecord::new(2, 900);
        record2.add_value("pressure", 1012.78);
        
        let records = vec![record1, record2];
        let processed = process_records(&records).unwrap();
        
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].timestamp, 900);
        assert_eq!(processed[1].timestamp, 1000);
    }

    #[test]
    fn test_calculate_statistics() {
        let mut record1 = DataRecord::new(1, 1000);
        record1.add_value("temperature", 22.5);
        record1.add_value("humidity", 65.0);
        
        let mut record2 = DataRecord::new(2, 1100);
        record2.add_value("temperature", 24.0);
        record2.add_value("humidity", 68.0);
        
        let records = vec![record1, record2];
        let stats = calculate_statistics(&records);
        
        assert_eq!(stats.get("temperature_avg"), Some(&23.25));
        assert_eq!(stats.get("humidity_min"), Some(&65.0));
    }
}