
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

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();

        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }

        groups
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.records.iter().map(|r| r.value).reduce(f64::max);

        (count, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let csv_data = "id,name,value,category\n1,ItemA,10.5,Alpha\n2,ItemB,15.0,Beta\n3,ItemC,20.3,Alpha";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        
        let valid_records = processor.validate_records();
        assert_eq!(valid_records.len(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.266).abs() < 0.001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.get("Alpha").unwrap().len(), 2);
        assert_eq!(groups.get("Beta").unwrap().len(), 1);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
        assert!((stats.1.unwrap() - 15.266).abs() < 0.001);
        assert!((stats.2.unwrap() - 20.3).abs() < 0.001);
    }
}
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: i64,
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid data value: {0}")]
    InvalidValue(f64),
    #[error("Timestamp out of range: {0}")]
    InvalidTimestamp(i64),
    #[error("ID must be positive: {0}")]
    InvalidId(u32),
}

pub struct DataProcessor {
    max_value: f64,
    min_timestamp: i64,
    max_timestamp: i64,
}

impl DataProcessor {
    pub fn new(max_value: f64, min_timestamp: i64, max_timestamp: i64) -> Self {
        Self {
            max_value,
            min_timestamp,
            max_timestamp,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId(record.id));
        }

        if record.value < 0.0 || record.value > self.max_value {
            return Err(DataError::InvalidValue(record.value));
        }

        if record.timestamp < self.min_timestamp || record.timestamp > self.max_timestamp {
            return Err(DataError::InvalidTimestamp(record.timestamp));
        }

        Ok(())
    }

    pub fn normalize_value(&self, record: &DataRecord) -> f64 {
        record.value / self.max_value
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, DataError>> {
        records
            .into_iter()
            .map(|record| {
                self.validate_record(&record)?;
                Ok(record)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let processor = DataProcessor::new(100.0, 0, 1000);
        let record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 500,
        };

        assert!(processor.validate_record(&record).is_ok());
        assert_eq!(processor.normalize_value(&record), 0.5);
    }

    #[test]
    fn test_invalid_value() {
        let processor = DataProcessor::new(100.0, 0, 1000);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            timestamp: 500,
        };

        assert!(matches!(
            processor.validate_record(&record),
            Err(DataError::InvalidValue(150.0))
        ));
    }
}