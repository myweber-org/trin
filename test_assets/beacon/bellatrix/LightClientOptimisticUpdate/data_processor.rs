
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
    InvalidName,
    InvalidValue,
    InvalidCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidName => write!(f, "Name cannot be empty"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::InvalidCategory => write!(f, "Category must be one of: A, B, C, D"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_stats: HashMap<String, (f64, usize)>,
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
            return Err(DataError::DuplicateRecord);
        }

        self.update_category_stats(&record, true);
        self.records.insert(record.id, record);
        
        Ok(())
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        if let Some(record) = self.records.remove(&id) {
            self.update_category_stats(&record, false);
            Some(record)
        } else {
            None
        }
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn calculate_average(&self, category: &str) -> Option<f64> {
        self.category_stats.get(category).map(|&(sum, count)| {
            if count > 0 { sum / count as f64 } else { 0.0 }
        })
    }

    pub fn transform_values<F>(&mut self, transformer: F) 
    where
        F: Fn(f64) -> f64,
    {
        let mut updates = Vec::new();
        
        for (id, record) in &mut self.records {
            let new_value = transformer(record.value);
            if (new_value - record.value).abs() > f64::EPSILON {
                let old_record = record.clone();
                record.value = new_value;
                updates.push((id, old_record, record.clone()));
            }
        }

        for (_, old_record, new_record) in updates {
            self.update_category_stats(&old_record, false);
            self.update_category_stats(&new_record, true);
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.values()
            .filter(|record| record.category == category)
            .collect()
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::InvalidName);
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

    fn update_category_stats(&mut self, record: &DataRecord, add: bool) {
        let entry = self.category_stats
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        
        if add {
            entry.0 += record.value;
            entry.1 += 1;
        } else {
            entry.0 -= record.value;
            entry.1 -= 1;
        }
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
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
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "X".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 50.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 100.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 150.0, category: "B".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.calculate_average("A"), Some(75.0));
        assert_eq!(processor.calculate_average("B"), Some(150.0));
        assert_eq!(processor.calculate_average("C"), None);
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };
        
        processor.add_record(record).unwrap();
        processor.transform_values(|x| x * 2.0);
        
        let updated = processor.get_record(1).unwrap();
        assert_eq!(updated.value, 100.0);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    details: String,
}

impl ValidationError {
    fn new(msg: &str) -> ValidationError {
        ValidationError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<DataRecord, ValidationError> {
        if id == 0 {
            return Err(ValidationError::new("ID cannot be zero"));
        }
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::new("Value must be between 0 and 1000"));
        }
        if timestamp < 0 {
            return Err(ValidationError::new("Timestamp cannot be negative"));
        }

        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }

    pub fn normalize(&self, max_value: f64) -> f64 {
        if max_value <= 0.0 {
            return 0.0;
        }
        self.value / max_value
    }

    pub fn is_anomaly(&self, threshold: f64) -> bool {
        self.value > threshold
    }
}

pub fn process_records(records: &[DataRecord]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = sum / count as f64;

    let max_value = records
        .iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);

    (average, max_value, count)
}

pub fn filter_records(records: &[DataRecord], min_value: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|r| r.value >= min_value)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, 1625097600);
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.timestamp, 1625097600);
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 500.0, 1625097600);
        assert!(record.is_err());
    }

    #[test]
    fn test_normalize() {
        let record = DataRecord::new(1, 250.0, 1625097600).unwrap();
        let normalized = record.normalize(500.0);
        assert_eq!(normalized, 0.5);
    }

    #[test]
    fn test_anomaly_detection() {
        let record = DataRecord::new(1, 750.0, 1625097600).unwrap();
        assert!(record.is_anomaly(700.0));
        assert!(!record.is_anomaly(800.0));
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 100.0, 1625097600).unwrap(),
            DataRecord::new(2, 200.0, 1625097601).unwrap(),
            DataRecord::new(3, 300.0, 1625097602).unwrap(),
        ];

        let (avg, max, count) = process_records(&records);
        assert_eq!(avg, 200.0);
        assert_eq!(max, 300.0);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            DataRecord::new(1, 50.0, 1625097600).unwrap(),
            DataRecord::new(2, 150.0, 1625097601).unwrap(),
            DataRecord::new(3, 250.0, 1625097602).unwrap(),
        ];

        let filtered = filter_records(&records, 100.0);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 2);
        assert_eq!(filtered[1].id, 3);
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
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
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

            if let Ok(record) = DataRecord::new(id, value, parts[2]) {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
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

    pub fn get_statistics(&self) -> Statistics {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len();

        if count == 0 {
            return Statistics::empty();
        }

        let sum: f64 = values.iter().sum();
        let average = sum / count as f64;
        let max = values.iter().fold(f64::MIN, |a, &b| a.max(b));
        let min = values.iter().fold(f64::MAX, |a, &b| a.min(b));

        Statistics {
            count,
            average,
            max,
            min,
        }
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub count: usize,
    pub average: f64,
    pub max: f64,
    pub min: f64,
}

impl Statistics {
    fn empty() -> Self {
        Self {
            count: 0,
            average: 0.0,
            max: 0.0,
            min: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.0,beta").unwrap();
        writeln!(temp_file, "3,15.75,alpha").unwrap();

        let mut processor = DataProcessor::new();
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.calculate_average(), Some(15.416666666666666));
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A").unwrap());
        processor.records.push(DataRecord::new(2, 20.0, "B").unwrap());
        processor.records.push(DataRecord::new(3, 30.0, "A").unwrap());

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }
}