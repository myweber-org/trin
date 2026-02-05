
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        let validated_data = self.validate_data(data)?;
        let transformed_data = self.transform_data(&validated_data);
        
        self.cache.insert(dataset_name.to_string(), transformed_data.clone());
        
        Ok(transformed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        for rule in &self.validation_rules {
            if rule.required && data.is_empty() {
                return Err(format!("Field {} is required but empty", rule.field_name));
            }
            
            for &value in data {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!("Value {} out of range for field {}", value, rule.field_name));
                }
            }
        }
        
        Ok(data.to_vec())
    }

    fn transform_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = self.calculate_mean(data);
        let std_dev = self.calculate_std_dev(data, mean);
        
        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn calculate_mean(&self, data: &[f64]) -> f64 {
        data.iter().sum::<f64>() / data.len() as f64
    }

    fn calculate_std_dev(&self, data: &[f64], mean: f64) -> f64 {
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        variance.sqrt()
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("temperature", -50.0, 100.0, true));
        
        let data = vec![20.5, 25.3, 18.7, 22.1];
        let result = processor.process_dataset("weather", &data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("weather").unwrap().len(), 4);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("pressure", 950.0, 1050.0, true));
        
        let invalid_data = vec![920.5, 1020.3];
        let result = processor.process_dataset("pressure_data", &invalid_data);
        
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
    pub timestamp: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String, timestamp: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() &&
        self.value >= 0.0 &&
        !self.category.is_empty() &&
        !self.timestamp.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<Record>,
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
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        let mut count = 0;
        for result in csv_reader.deserialize() {
            let record: Record = result?;
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
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

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn add_record(&mut self, record: Record) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn get_records(&self) -> &[Record] {
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
    fn test_record_validation() {
        let valid_record = Record::new(
            1,
            "Test".to_string(),
            42.5,
            "CategoryA".to_string(),
            "2024-01-15T10:30:00Z".to_string(),
        );
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(
            2,
            "".to_string(),
            -10.0,
            "".to_string(),
            "".to_string(),
        );
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record1 = Record::new(
            1,
            "Item1".to_string(),
            100.0,
            "CategoryA".to_string(),
            "2024-01-15T10:30:00Z".to_string(),
        );
        
        let record2 = Record::new(
            2,
            "Item2".to_string(),
            200.0,
            "CategoryB".to_string(),
            "2024-01-15T11:30:00Z".to_string(),
        );

        processor.add_record(record1.clone());
        processor.add_record(record2.clone());

        assert_eq!(processor.get_records().len(), 2);
        
        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);

        let avg = processor.calculate_average_value();
        assert_eq!(avg, Some(150.0));

        processor.clear();
        assert_eq!(processor.get_records().len(), 0);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        
        let record = Record::new(
            1,
            "TestItem".to_string(),
            123.45,
            "TestCategory".to_string(),
            "2024-01-15T12:00:00Z".to_string(),
        );
        
        processor.add_record(record);

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        processor.save_to_csv(path)?;

        let mut new_processor = DataProcessor::new();
        let count = new_processor.load_from_csv(path)?;
        
        assert_eq!(count, 1);
        assert_eq!(new_processor.get_records().len(), 1);
        
        Ok(())
    }
}