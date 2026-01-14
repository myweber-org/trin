use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: u64) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
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
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut rdr = csv::Reader::from_reader(reader);
        let mut count = 0;

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
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
        let mut wtr = csv::Writer::from_writer(writer);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_stats(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test".to_string(), 1234567890);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, 42.5, "test".to_string(), 1234567890);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert!(processor.is_empty());

        processor.add_record(DataRecord::new(1, 10.0, "A".to_string(), 1000));
        processor.add_record(DataRecord::new(2, 20.0, "A".to_string(), 2000));
        processor.add_record(DataRecord::new(3, 30.0, "B".to_string(), 3000));

        assert_eq!(processor.len(), 3);
        assert_eq!(processor.filter_by_category("A").len(), 2);
        assert_eq!(processor.calculate_average(), Some(20.0));

        let (min, max, avg) = processor.get_stats();
        assert_eq!(min, 10.0);
        assert_eq!(max, 30.0);
        assert_eq!(avg, 20.0);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 10.0, "test".to_string(), 1000));
        processor.add_record(DataRecord::new(2, 20.0, "test".to_string(), 2000));

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        processor.save_to_csv(path)?;

        let mut new_processor = DataProcessor::new();
        let count = new_processor.load_from_csv(path)?;

        assert_eq!(count, 2);
        assert_eq!(new_processor.len(), 2);
        Ok(())
    }
}
use std::collections::HashMap;
use std::error::Error;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn register_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn register_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn process_data(&self, data: &str, validator_name: &str, transformer_name: &str) -> Result<String, Box<dyn Error>> {
        let validator = self.validators.get(validator_name)
            .ok_or_else(|| format!("Validator '{}' not found", validator_name))?;

        if !validator(data) {
            return Err(format!("Validation failed for data: {}", data).into());
        }

        let transformer = self.transformers.get(transformer_name)
            .ok_or_else(|| format!("Transformer '{}' not found", transformer_name))?;

        Ok(transformer(data.to_string()))
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.register_validator("is_numeric", Box::new(|s| s.chars().all(|c| c.is_ascii_digit())));

    processor.register_validator("is_alpha", Box::new(|s| s.chars().all(|c| c.is_ascii_alphabetic())));

    processor.register_transformer("uppercase", Box::new(|s| s.to_uppercase()));

    processor.register_transformer("reverse", Box::new(|s| s.chars().rev().collect()));

    processor.register_transformer("add_prefix", Box::new(|s| format!("processed_{}", s)));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_validation() {
        let processor = create_default_processor();
        assert!(processor.process_data("12345", "is_numeric", "uppercase").is_ok());
        assert!(processor.process_data("abc123", "is_numeric", "uppercase").is_err());
    }

    #[test]
    fn test_alpha_validation() {
        let processor = create_default_processor();
        assert!(processor.process_data("hello", "is_alpha", "uppercase").is_ok());
        assert!(processor.process_data("hello123", "is_alpha", "uppercase").is_err());
    }

    #[test]
    fn test_transformations() {
        let processor = create_default_processor();
        
        let result = processor.process_data("test", "is_alpha", "uppercase").unwrap();
        assert_eq!(result, "TEST");

        let result = processor.process_data("hello", "is_alpha", "reverse").unwrap();
        assert_eq!(result, "olleh");

        let result = processor.process_data("data", "is_alpha", "add_prefix").unwrap();
        assert_eq!(result, "processed_data");
    }

    #[test]
    fn test_invalid_validator() {
        let processor = create_default_processor();
        assert!(processor.process_data("test", "non_existent", "uppercase").is_err());
    }

    #[test]
    fn test_invalid_transformer() {
        let processor = create_default_processor();
        assert!(processor.process_data("test", "is_alpha", "non_existent").is_err());
    }
}