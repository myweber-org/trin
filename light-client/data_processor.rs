
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: i64,
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
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let mut count = 0;
        for (line_num, line) in content.lines().enumerate() {
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

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let timestamp = match parts[3].parse::<i64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            self.records.push(DataRecord {
                id,
                value,
                category,
                timestamp,
            });
            count += 1;
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

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_records_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
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
        assert_eq!(processor.get_records_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,42.5,category_a,1625097600").unwrap();
        writeln!(temp_file, "2,37.8,category_b,1625184000").unwrap();
        writeln!(temp_file, "3,55.2,category_a,1625270400").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.get_records_count(), 3);

        let category_a_records = processor.filter_by_category("category_a");
        assert_eq!(category_a_records.len(), 2);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 45.166666666666664).abs() < 0.0001);

        processor.clear();
        assert_eq!(processor.get_records_count(), 0);
    }
}
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

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), DataError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, DataError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), DataError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, DataError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, DataError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, DataError>> {
        records.into_iter().map(|r| self.process(r)).collect()
    }
}

fn validate_timestamp(record: &DataRecord) -> Result<(), DataError> {
    if record.timestamp < 0 {
        return Err(DataError::ValidationFailed(
            "Timestamp cannot be negative".to_string(),
        ));
    }
    Ok(())
}

fn normalize_values(record: DataRecord) -> Result<DataRecord, DataError> {
    if record.values.is_empty() {
        return Err(DataError::TransformationError(
            "No values to normalize".to_string(),
        ));
    }

    let sum: f64 = record.values.iter().sum();
    if sum == 0.0 {
        return Ok(record);
    }

    let normalized_values: Vec<f64> = record
        .values
        .iter()
        .map(|&v| v / sum)
        .collect();

    Ok(DataRecord {
        values: normalized_values,
        ..record
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(validate_timestamp);
        processor.add_transformation(normalize_values);

        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(result.is_ok());

        let processed = result.unwrap();
        let sum: f64 = processed.values.iter().sum();
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_invalid_timestamp() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(validate_timestamp);

        let record = DataRecord {
            id: 2,
            timestamp: -1,
            values: vec![1.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(result.is_err());
    }
}