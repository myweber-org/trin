
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: String,
    pub value: f64,
    pub category: String,
    pub is_valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: String, value: f64, category: String) -> Self {
        let is_valid = value >= 0.0 && !category.is_empty();
        Self {
            id,
            timestamp,
            value,
            category,
            is_valid,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.value < 0.0 {
            return Err(format!("Invalid value {} for record {}", self.value, self.id));
        }
        if self.category.is_empty() {
            return Err(format!("Empty category for record {}", self.id));
        }
        Ok(())
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

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.is_valid)
            .collect()
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

    pub fn validate_all(&self) -> Vec<Result<(), String>> {
        self.records
            .iter()
            .map(|record| record.validate())
            .collect()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(
            1,
            "2024-01-15T10:30:00Z".to_string(),
            42.5,
            "temperature".to_string(),
        );
        
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "temperature");
        assert!(record.is_valid);
    }

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(
            1,
            "2024-01-15T10:30:00Z".to_string(),
            42.5,
            "temperature".to_string(),
        );
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(
            2,
            "2024-01-15T10:30:00Z".to_string(),
            -5.0,
            "pressure".to_string(),
        );
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(
            1,
            "2024-01-15T10:30:00Z".to_string(),
            42.5,
            "temperature".to_string(),
        );
        
        let record2 = DataRecord::new(
            2,
            "2024-01-15T11:00:00Z".to_string(),
            25.0,
            "humidity".to_string(),
        );
        
        processor.add_record(record1);
        processor.add_record(record2);
        
        assert_eq!(processor.count(), 2);
        assert_eq!(processor.filter_valid().len(), 2);
        assert_eq!(processor.filter_by_category("temperature").len(), 1);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), 33.75);
    }

    #[test]
    fn test_csv_operations() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord::new(
            1,
            "2024-01-15T10:30:00Z".to_string(),
            42.5,
            "temperature".to_string(),
        );
        
        processor.add_record(record);
        
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        
        assert!(processor.save_to_csv(path).is_ok());
        
        let mut new_processor = DataProcessor::new();
        assert!(new_processor.load_from_csv(path).is_ok());
        assert_eq!(new_processor.count(), 1);
    }
}