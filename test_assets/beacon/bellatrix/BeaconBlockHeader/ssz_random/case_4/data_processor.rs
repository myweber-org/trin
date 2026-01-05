
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
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

            let category = parts[2].trim().to_string();
            let record = DataRecord::new(id, value, category);
            
            if record.is_valid() {
                self.total_value += record.get_value();
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.total_value / self.records.len() as f64)
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, f64, Option<f64>) {
        let count = self.records.len();
        let total = self.total_value;
        let average = self.get_average_value();
        (count, total, average)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_value = DataRecord::new(2, -5.0, "B".to_string());
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(3, 15.0, "".to_string());
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,20.0,TypeB").unwrap();
        writeln!(temp_file, "3,15.5,TypeA").unwrap();
        writeln!(temp_file, "4,-5.0,TypeC").unwrap();
        
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        
        let average = processor.get_average_value().unwrap();
        assert!((average - 15.33333).abs() < 0.0001);
        
        let type_a_records = processor.filter_by_category("TypeA");
        assert_eq!(type_a_records.len(), 2);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
        assert_eq!(stats.1, 46.0);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessingError {
    details: String,
}

impl ProcessingError {
    fn new(msg: &str) -> ProcessingError {
        ProcessingError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ProcessingError {}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, ProcessingError> {
        if id == 0 {
            return Err(ProcessingError::new("ID cannot be zero"));
        }
        if !value.is_finite() {
            return Err(ProcessingError::new("Value must be finite"));
        }
        if timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }

        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }

    pub fn transform(&self, multiplier: f64) -> Result<f64, ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::new("Multiplier must be positive"));
        }
        Ok(self.value * multiplier)
    }
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<f64>, ProcessingError> {
    if records.is_empty() {
        return Err(ProcessingError::new("No records to process"));
    }

    let mut results = Vec::with_capacity(records.len());
    for record in records {
        let transformed = record.transform(2.5)?;
        results.push(transformed);
    }

    Ok(results)
}

pub fn calculate_statistics(values: &[f64]) -> (f64, f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = values.iter().sum();
    let count = values.len() as f64;
    let mean = sum / count;

    let variance: f64 = values
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>()
        / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, 1234567890).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.timestamp, 1234567890);
    }

    #[test]
    fn test_invalid_record_id() {
        let result = DataRecord::new(0, 42.5, 1234567890);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord::new(1, 10.0, 1234567890).unwrap();
        let transformed = record.transform(3.0).unwrap();
        assert_eq!(transformed, 30.0);
    }

    #[test]
    fn test_process_multiple_records() {
        let records = vec![
            DataRecord::new(1, 10.0, 1000).unwrap(),
            DataRecord::new(2, 20.0, 2000).unwrap(),
            DataRecord::new(3, 30.0, 3000).unwrap(),
        ];

        let results = process_records(&records).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], 25.0);
        assert_eq!(results[1], 50.0);
        assert_eq!(results[2], 75.0);
    }

    #[test]
    fn test_calculate_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = calculate_statistics(&values);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
    }
}