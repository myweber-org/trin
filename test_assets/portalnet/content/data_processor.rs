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

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid ID: zero is not allowed".to_string());
        }
        if self.value < 0.0 {
            return Err("Invalid value: negative numbers not allowed".to_string());
        }
        if self.category.is_empty() {
            return Err("Invalid category: empty string".to_string());
        }
        Ok(())
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

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(-1.0);
            let category = parts[2].trim();

            let record = DataRecord::new(id, value, category);
            if record.valid {
                self.records.push(record);
                count += 1;
            }
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

    pub fn statistics(&self) -> (usize, usize, Option<f64>) {
        let total = self.records.len();
        let valid_count = self.filter_valid().len();
        let avg = self.calculate_average();
        (total, valid_count, avg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test");
        assert!(valid_record.validate().is_ok());
        assert!(valid_record.valid);

        let invalid_record = DataRecord::new(0, -1.0, "");
        assert!(invalid_record.validate().is_err());
        assert!(!invalid_record.valid);
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, 10.0, "A"));
        processor.add_record(DataRecord::new(2, 20.0, "B"));
        processor.add_record(DataRecord::new(3, -5.0, "C"));
        
        let (total, valid, avg) = processor.statistics();
        assert_eq!(total, 3);
        assert_eq!(valid, 2);
        assert_eq!(avg, Some(15.0));
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("A").unwrap().len(), 1);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: u64,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue,
    MissingField,
    ParseError(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue => write!(f, "Invalid data value"),
            DataError::MissingField => write!(f, "Missing required field"),
            DataError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue);
        }
        
        if record.value < 0.0 || record.value > self.threshold {
            return Err(DataError::InvalidValue);
        }
        
        Ok(())
    }

    pub fn transform_value(&self, record: &DataRecord) -> f64 {
        (record.value * 100.0).round() / 100.0
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, DataError>> {
        records
            .into_iter()
            .map(|mut record| {
                self.validate_record(&record)?;
                record.value = self.transform_value(&record);
                Ok(record)
            })
            .collect()
    }
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
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
    fn test_validate_valid_record() {
        let processor = DataProcessor::new(1000.0);
        let record = DataRecord {
            id: 1,
            value: 500.0,
            timestamp: 1234567890,
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_invalid_record() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 1234567890,
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_value() {
        let processor = DataProcessor::new(1000.0);
        let record = DataRecord {
            id: 1,
            value: 123.456789,
            timestamp: 1234567890,
        };
        
        let transformed = processor.transform_value(&record);
        assert_eq!(transformed, 123.46);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1 },
            DataRecord { id: 2, value: 20.0, timestamp: 2 },
            DataRecord { id: 3, value: 30.0, timestamp: 3 },
        ];
        
        let avg = calculate_average(&records);
        assert_eq!(avg, Some(20.0));
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
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyName,
    InvalidCategory,
    DuplicateRecord(u32),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::InvalidCategory => write!(f, "Category must be one of: A, B, C, D"),
            DataError::DuplicateRecord(id) => write!(f, "Duplicate record with ID: {}", id),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub count: usize,
    pub total_value: f64,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord(record.id));
        }
        
        self.records.insert(record.id, record.clone());
        self.update_category_stats(&record);
        
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        let record = self.records.remove(&id);
        if let Some(ref rec) = record {
            self.recalculate_category_stats();
        }
        record
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn filter_by_value_range(&self, min: f64, max: f64) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.value >= min && record.value <= max)
            .collect()
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.values().map(|record| record.value).sum()
    }

    pub fn get_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }

    pub fn transform_records<F>(&mut self, transform_fn: F) 
    where
        F: Fn(&DataRecord) -> DataRecord,
    {
        let transformed: Vec<DataRecord> = self.records
            .values()
            .map(|record| transform_fn(record))
            .collect();
        
        self.records.clear();
        self.category_stats.clear();
        
        for record in transformed {
            let _ = self.add_record(record);
        }
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        
        if !(0.0..=1000.0).contains(&record.value) {
            return Err(DataError::InvalidValue);
        }
        
        let valid_categories = ["A", "B", "C", "D"];
        if !valid_categories.contains(&record.category.as_str()) {
            return Err(DataError::InvalidCategory);
        }
        
        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                count: 0,
                total_value: 0.0,
                avg_value: 0.0,
                min_value: f64::MAX,
                max_value: f64::MIN,
            });
        
        stats.count += 1;
        stats.total_value += record.value;
        stats.avg_value = stats.total_value / stats.count as f64;
        stats.min_value = stats.min_value.min(record.value);
        stats.max_value = stats.max_value.max(record.value);
    }

    fn recalculate_category_stats(&mut self) {
        self.category_stats.clear();
        
        for record in self.records.values() {
            self.update_category_stats(record);
        }
    }
}

pub fn normalize_value(value: f64, min: f64, max: f64) -> f64 {
    if max == min {
        return 0.5;
    }
    (value - min) / (max - min)
}

pub fn create_sample_record(id: u32, name: &str, value: f64, category: &str) -> DataRecord {
    DataRecord {
        id,
        name: name.to_string(),
        value,
        category: category.to_string(),
        tags: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = create_sample_record(1, "Test", 100.0, "A");
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = create_sample_record(1, "Test1", 100.0, "A");
        let record2 = create_sample_record(1, "Test2", 200.0, "B");
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(create_sample_record(1, "Test1", 100.0, "A")).unwrap();
        processor.add_record(create_sample_record(2, "Test2", 200.0, "B")).unwrap();
        processor.add_record(create_sample_record(3, "Test3", 300.0, "A")).unwrap();
        
        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);
    }

    #[test]
    fn test_normalize_value() {
        assert_eq!(normalize_value(50.0, 0.0, 100.0), 0.5);
        assert_eq!(normalize_value(75.0, 50.0, 100.0), 0.5);
        assert_eq!(normalize_value(0.0, 0.0, 100.0), 0.0);
        assert_eq!(normalize_value(100.0, 0.0, 100.0), 1.0);
    }
}