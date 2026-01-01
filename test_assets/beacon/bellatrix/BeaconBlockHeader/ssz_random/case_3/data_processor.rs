use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        Self {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.values.is_empty() {
            return Err(DataError::ValidationFailed("Empty values array".into()));
        }

        let sum: f64 = record.values.iter().sum();
        let avg = sum / record.values.len() as f64;

        if avg.abs() > self.validation_threshold {
            return Err(DataError::ValidationFailed(
                format!("Average value {} exceeds threshold {}", avg, self.validation_threshold)
            ));
        }

        if record.timestamp < 0 {
            return Err(DataError::ValidationFailed("Negative timestamp".into()));
        }

        Ok(())
    }

    pub fn transform_record(&self, mut record: DataRecord) -> Result<DataRecord, DataError> {
        self.validate_record(&record)?;

        for value in record.values.iter_mut() {
            *value *= self.transformation_factor;
        }

        record.metadata.insert(
            "processed".into(),
            chrono::Utc::now().to_rfc3339()
        );

        Ok(record)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>
    ) -> Result<Vec<DataRecord>, (Vec<DataRecord>, DataError)> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            match self.transform_record(record) {
                Ok(transformed) => processed.push(transformed),
                Err(e) => return Err((processed, e)),
            }
        }
        
        Ok(processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(100.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(10.0, 2.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![100.0, 200.0, 300.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(100.0, 3.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        let transformed = processor.transform_record(record).unwrap();
        assert_eq!(transformed.values, vec![3.0, 6.0, 9.0]);
        assert!(transformed.metadata.contains_key("processed"));
    }
}
use std::error::Error;
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
            self.records.push(record);
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

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && r.value <= 1000.0)
            .collect()
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
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
        writeln!(temp_file, "1,100.5,alpha").unwrap();
        writeln!(temp_file, "2,200.3,beta").unwrap();
        writeln!(temp_file, "3,300.7,alpha").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 200.5).abs() < 0.1);

        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);

        let stats = processor.get_statistics();
        assert!((stats.0 - 100.5).abs() < 0.1);
        assert!((stats.1 - 300.7).abs() < 0.1);
    }
}