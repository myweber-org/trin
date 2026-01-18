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