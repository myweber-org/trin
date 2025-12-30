
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        Self {
            id,
            value,
            category: category.to_string(),
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
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, parts[2]);
            if record.is_valid() {
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
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test");
        assert!(valid_record.is_valid());

        let invalid_id = DataRecord::new(0, 42.5, "test");
        assert!(!invalid_id.is_valid());

        let invalid_value = DataRecord::new(1, f64::NAN, "test");
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(1, 42.5, "");
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,invalid,category_c").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test"));
        processor.records.push(DataRecord::new(2, 20.0, "test"));
        processor.records.push(DataRecord::new(3, 30.0, "test"));

        assert_eq!(processor.calculate_average(), Some(20.0));
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.calculate_average(), None);
        assert_eq!(processor.get_statistics(), (0.0, 0.0, 0.0));
    }
}
use std::collections::HashMap;

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

    pub fn add_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn add_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn validate(&self, name: &str, data: &str) -> bool {
        self.validators
            .get(name)
            .map_or(false, |validator| validator(data))
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers
            .get(name)
            .map(|transformer| transformer(data))
    }

    pub fn process(&self, validator_name: &str, transformer_name: &str, data: String) -> Option<String> {
        if self.validate(validator_name, &data) {
            self.transform(transformer_name, data)
        } else {
            None
        }
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_validator("is_numeric", Box::new(|s| s.chars().all(|c| c.is_ascii_digit())));
    processor.add_validator("is_alpha", Box::new(|s| s.chars().all(|c| c.is_ascii_alphabetic())));
    
    processor.add_transformer("to_uppercase", Box::new(|s| s.to_uppercase()));
    processor.add_transformer("reverse", Box::new(|s| s.chars().rev().collect()));
    
    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("is_numeric", "12345"));
        assert!(!processor.validate("is_numeric", "123a5"));
    }

    #[test]
    fn test_alpha_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("is_alpha", "hello"));
        assert!(!processor.validate("is_alpha", "hello123"));
    }

    #[test]
    fn test_uppercase_transformation() {
        let processor = create_default_processor();
        let result = processor.transform("to_uppercase", "hello".to_string());
        assert_eq!(result, Some("HELLO".to_string()));
    }

    #[test]
    fn test_reverse_transformation() {
        let processor = create_default_processor();
        let result = processor.transform("reverse", "hello".to_string());
        assert_eq!(result, Some("olleh".to_string()));
    }

    #[test]
    fn test_full_processing() {
        let processor = create_default_processor();
        let result = processor.process("is_alpha", "to_uppercase", "hello".to_string());
        assert_eq!(result, Some("HELLO".to_string()));
        
        let invalid_result = processor.process("is_numeric", "to_uppercase", "hello".to_string());
        assert_eq!(invalid_result, None);
    }
}