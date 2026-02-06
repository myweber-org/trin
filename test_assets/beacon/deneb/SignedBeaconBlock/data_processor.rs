use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
    pub timestamp: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in &self.records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn validate_records(&self) -> Vec<(usize, String)> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if record.name.trim().is_empty() {
                errors.push((index, "Name cannot be empty".to_string()));
            }

            if record.value < 0.0 {
                errors.push((index, "Value cannot be negative".to_string()));
            }

            if record.category.trim().is_empty() {
                errors.push((index, "Category cannot be empty".to_string()));
            }
        }

        errors
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 42.5,
            category: "Test".to_string(),
            timestamp: "2024-01-15T10:30:00Z".to_string(),
        };

        processor.add_record(record.clone());

        assert_eq!(processor.get_records().len(), 1);
        assert_eq!(processor.calculate_average(), Some(42.5));

        let filtered = processor.filter_by_category("Test");
        assert_eq!(filtered.len(), 1);

        let errors = processor.validate_records();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_csv_operations() {
        let mut processor = DataProcessor::new();

        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 10.0,
            category: "A".to_string(),
            timestamp: "2024-01-15T10:30:00Z".to_string(),
        };

        let record2 = DataRecord {
            id: 2,
            name: "Second".to_string(),
            value: 20.0,
            category: "B".to_string(),
            timestamp: "2024-01-15T11:30:00Z".to_string(),
        };

        processor.add_record(record1);
        processor.add_record(record2);

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        processor.save_to_csv(path).unwrap();

        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path).unwrap();

        assert_eq!(new_processor.get_records().len(), 2);
        assert_eq!(new_processor.calculate_average(), Some(15.0));
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
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("Empty values array")]
    EmptyValues,
    #[error("NaN value detected at index {0}")]
    NaNValue(usize),
    #[error("Duplicate record ID: {0}")]
    DuplicateId(u64),
}

pub struct DataProcessor {
    processed_ids: std::collections::HashSet<u64>,
    stats: ProcessingStats,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_records: u64,
    pub valid_records: u64,
    pub invalid_records: u64,
    pub total_values: u64,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            processed_ids: std::collections::HashSet::new(),
            stats: ProcessingStats::default(),
        }
    }

    pub fn process_record(&mut self, record: &DataRecord) -> Result<(), DataError> {
        self.stats.total_records += 1;

        if self.processed_ids.contains(&record.id) {
            self.stats.invalid_records += 1;
            return Err(DataError::DuplicateId(record.id));
        }

        self.validate_record(record)?;

        self.processed_ids.insert(record.id);
        self.stats.valid_records += 1;
        self.stats.total_values += record.values.len() as u64;

        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.timestamp < 0 {
            return Err(DataError::InvalidTimestamp(record.timestamp));
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for (index, &value) in record.values.iter().enumerate() {
            if value.is_nan() {
                return Err(DataError::NaNValue(index));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, values: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;

        if range.abs() < f64::EPSILON {
            return vec![0.0; values.len()];
        }

        values
            .iter()
            .map(|&v| (v - min) / range)
            .collect()
    }

    pub fn get_stats(&self) -> &ProcessingStats {
        &self.stats
    }

    pub fn reset(&mut self) {
        self.processed_ids.clear();
        self.stats = ProcessingStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.process_record(&record).is_ok());
        assert_eq!(processor.get_stats().valid_records, 1);
        assert_eq!(processor.get_stats().total_values, 3);
    }

    #[test]
    fn test_duplicate_id() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0],
            metadata: HashMap::new(),
        };

        assert!(processor.process_record(&record).is_ok());
        assert!(processor.process_record(&record).is_err());
        assert_eq!(processor.get_stats().invalid_records, 1);
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = processor.normalize_values(&values);

        assert_eq!(normalized[0], 0.0);
        assert_eq!(normalized[4], 1.0);
        assert!(normalized[2] > 0.4 && normalized[2] < 0.6);
    }
}