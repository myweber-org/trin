
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
        let variance: f64 = values.iter()
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
            .map(|&x| x.powi(2).ln().max(0.0))
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_items = self.cache.len();
        let total_values: usize = self.cache.values().map(|v| v.len()).sum();
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

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let _ = processor.process_numeric_data("cached", &data);
        let (items, values) = processor.get_cache_stats();
        
        assert_eq!(items, 1);
        assert_eq!(values, 3);
        
        processor.clear_cache();
        let (items, _) = processor.get_cache_stats();
        assert_eq!(items, 0);
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
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    MissingField,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be positive"),
            DataError::MissingField => write!(f, "Required field is missing"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_totals: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_totals: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.value <= 0.0 {
            return Err(DataError::InvalidValue);
        }

        if record.name.is_empty() || record.category.is_empty() {
            return Err(DataError::MissingField);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        let category_total = self.category_totals
            .entry(record.category.clone())
            .or_insert(0.0);
        *category_total += record.value;

        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_total(&self, category: &str) -> f64 {
        *self.category_totals.get(category).unwrap_or(&0.0)
    }

    pub fn apply_discount(&mut self, category: &str, discount_percent: f64) -> Result<(), DataError> {
        if discount_percent < 0.0 || discount_percent > 100.0 {
            return Err(DataError::InvalidValue);
        }

        let multiplier = 1.0 - (discount_percent / 100.0);
        
        for record in self.records.values_mut() {
            if record.category == category {
                record.value *= multiplier;
            }
        }

        if let Some(total) = self.category_totals.get_mut(category) {
            *total *= multiplier;
        }

        Ok(())
    }

    pub fn get_all_records(&self) -> Vec<&DataRecord> {
        self.records.values().collect()
    }

    pub fn get_records_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let total: f64 = self.records.values().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        if let Some(record) = self.records.remove(&id) {
            if let Some(total) = self.category_totals.get_mut(&record.category) {
                *total -= record.value;
                if *total <= 0.0 {
                    self.category_totals.remove(&record.category);
                }
            }
            Some(record)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.category_totals.clear();
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_err());
        assert_eq!(processor.record_count(), 0);
    }

    #[test]
    fn test_category_total() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Item1".to_string(),
            value: 50.0,
            category: "Electronics".to_string(),
        };

        let record2 = DataRecord {
            id: 2,
            name: "Item2".to_string(),
            value: 75.0,
            category: "Electronics".to_string(),
        };

        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();

        assert_eq!(processor.get_category_total("Electronics"), 125.0);
    }

    #[test]
    fn test_apply_discount() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Item".to_string(),
            value: 100.0,
            category: "Sale".to_string(),
        };

        processor.add_record(record).unwrap();
        processor.apply_discount("Sale", 20.0).unwrap();

        let updated_record = processor.get_record(1).unwrap();
        assert_eq!(updated_record.value, 80.0);
        assert_eq!(processor.get_category_total("Sale"), 80.0);
    }

    #[test]
    fn test_remove_record() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        processor.add_record(record).unwrap();
        assert_eq!(processor.record_count(), 1);
        
        let removed = processor.remove_record(1);
        assert!(removed.is_some());
        assert_eq!(processor.record_count(), 0);
        assert_eq!(processor.get_category_total("A"), 0.0);
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.validate_record(&record)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), String> {
        if record.id == 0 {
            return Err("Invalid ID: zero is not allowed".to_string());
        }
        if record.value < 0.0 {
            return Err("Invalid value: negative numbers not allowed".to_string());
        }
        if record.category.is_empty() {
            return Err("Invalid category: empty string".to_string());
        }
        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
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
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,42.5,TypeA").unwrap();
        writeln!(temp_file, "2,33.7,TypeB").unwrap();
        writeln!(temp_file, "3,55.1,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 43.766).abs() < 0.001);
        
        let filtered = processor.filter_by_category("TypeA");
        assert_eq!(filtered.len(), 2);
    }
}use std::error::Error;
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

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let value = parts[1].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let category = parts[2].trim().to_string();

        match DataRecord::new(id, value, category) {
            Ok(record) => records.push(record),
            Err(e) => return Err(format!("Validation error at line {}: {}", line_number, e).into()),
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_negative_value() {
        let record = DataRecord::new(1, -5.0, "test".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_empty_category() {
        let record = DataRecord::new(1, 5.0, "".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_calculate_adjusted_value() {
        let record = DataRecord::new(1, 10.0, "test".to_string()).unwrap();
        assert_eq!(record.calculate_adjusted_value(2.5), 25.0);
    }

    #[test]
    fn test_process_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.0,category_b").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,30.75,category_c").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].calculate_adjusted_value(1.0), 10.5);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 10.0, "a".to_string()).unwrap(),
            DataRecord::new(2, 20.0, "b".to_string()).unwrap(),
            DataRecord::new(3, 30.0, "c".to_string()).unwrap(),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_empty_statistics() {
        let records: Vec<DataRecord> = vec![];
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 0.0);
        assert_eq!(variance, 0.0);
        assert_eq!(std_dev, 0.0);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessingError {
    message: String,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Processing error: {}", self.message)
    }
}

impl Error for ProcessingError {}

impl ProcessingError {
    pub fn new(msg: &str) -> Self {
        ProcessingError {
            message: msg.to_string(),
        }
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::new("ID cannot be zero"));
        }
        
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(ProcessingError::new("Value must be a finite number"));
        }
        
        if self.timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::new("Multiplier must be positive"));
        }
        
        self.value *= multiplier;
        Ok(())
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(multiplier)?;
        processed.push(DataRecord {
            id: record.id,
            value: record.value,
            timestamp: record.timestamp,
        });
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1234567890,
        };
        
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_transform_record() {
        let mut record = DataRecord {
            id: 1,
            value: 10.0,
            timestamp: 1234567890,
        };
        
        assert!(record.transform(2.5).is_ok());
        assert_eq!(record.value, 25.0);
    }
}