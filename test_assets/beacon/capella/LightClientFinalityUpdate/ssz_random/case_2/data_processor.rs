
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
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
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 3 {
                if let (Ok(id), Ok(value), timestamp) = (
                    parts[0].parse::<u32>(),
                    parts[1].parse::<f64>(),
                    parts[2].to_string(),
                ) {
                    self.records.push(DataRecord {
                        id,
                        value,
                        timestamp,
                    });
                    count += 1;
                }
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

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "id,value,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,2023-01-01T12:00:00").unwrap();
        writeln!(temp_file, "2,20.3,2023-01-01T12:05:00").unwrap();
        writeln!(temp_file, "3,15.7,2023-01-01T12:10:00").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.record_count(), 3);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.01);

        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error for field '{}': {}", self.field, self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> Result<(), ValidationError>>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_validator<F>(&mut self, field: &str, validator: F)
    where
        F: Fn(&str) -> Result<(), ValidationError> + 'static,
    {
        self.validators.insert(field.to_string(), Box::new(validator));
    }

    pub fn add_transformer<F>(&mut self, field: &str, transformer: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformers.insert(field.to_string(), Box::new(transformer));
    }

    pub fn process_data(&self, data: &mut HashMap<String, String>) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (field, value) in data.iter() {
            if let Some(validator) = self.validators.get(field) {
                if let Err(err) = validator(value) {
                    errors.push(err);
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        for (field, value) in data.iter_mut() {
            if let Some(transformer) = self.transformers.get(field) {
                *value = transformer(value.clone());
            }
        }

        Ok(())
    }
}

pub fn create_email_validator() -> impl Fn(&str) -> Result<(), ValidationError> {
    move |email: &str| {
        if email.contains('@') && email.contains('.') {
            Ok(())
        } else {
            Err(ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            })
        }
    }
}

pub fn create_name_transformer() -> impl Fn(String) -> String {
    move |name: String| {
        let mut chars = name.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = create_email_validator();
        assert!(validator("test@example.com").is_ok());
        assert!(validator("invalid-email").is_err());
    }

    #[test]
    fn test_name_transformation() {
        let transformer = create_name_transformer();
        assert_eq!(transformer("john".to_string()), "John");
        assert_eq!(transformer("JANE".to_string()), "JANE");
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validator("email", create_email_validator());
        processor.add_transformer("name", create_name_transformer());

        let mut data = HashMap::new();
        data.insert("email".to_string(), "test@example.com".to_string());
        data.insert("name".to_string(), "john doe".to_string());

        let result = processor.process_data(&mut data);
        assert!(result.is_ok());
        assert_eq!(data.get("name").unwrap(), "John doe");
    }
}