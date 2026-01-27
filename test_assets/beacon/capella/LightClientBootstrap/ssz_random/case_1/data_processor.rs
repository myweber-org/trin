use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
                
                let category = parts[0].to_string();
                *self.frequency_map.entry(category).or_insert(0) += 1;
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(15.5));
        assert_eq!(processor.calculate_median(), Some(15.7));
        
        let filtered = processor.filter_by_threshold(12.0);
        assert_eq!(filtered.len(), 2);
        
        let freq = processor.get_frequency_distribution();
        assert_eq!(freq.get("A"), Some(&2));
        assert_eq!(freq.get("B"), Some(&1));
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

    pub fn validate(&self, name: &str, data: &str) -> Option<bool> {
        self.validators.get(name).map(|validator| validator(data))
    }

    pub fn transform(&self, name: &str, data: String) -> Option<String> {
        self.transformers.get(name).map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: &str, validators: &[&str], transformers: &[&str]) -> Result<String, String> {
        let mut result = data.to_string();

        for validator_name in validators {
            if let Some(validator) = self.validators.get(*validator_name) {
                if !validator(&result) {
                    return Err(format!("Validation failed for: {}", validator_name));
                }
            } else {
                return Err(format!("Validator not found: {}", validator_name));
            }
        }

        for transformer_name in transformers {
            if let Some(transformer) = self.transformers.get(*transformer_name) {
                result = transformer(result);
            } else {
                return Err(format!("Transformer not found: {}", transformer_name));
            }
        }

        Ok(result)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validator("not_empty", Box::new(|s: &str| !s.trim().is_empty()));
    processor.add_validator("is_numeric", Box::new(|s: &str| s.chars().all(|c| c.is_ascii_digit())));

    processor.add_transformer("trim", Box::new(|s: String| s.trim().to_string()));
    processor.add_transformer("uppercase", Box::new(|s: String| s.to_uppercase()));
    processor.add_transformer("reverse", Box::new(|s: String| s.chars().rev().collect()));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        assert_eq!(processor.validate("not_empty", "test"), Some(true));
        assert_eq!(processor.validate("not_empty", ""), Some(false));
        assert_eq!(processor.validate("is_numeric", "123"), Some(true));
        assert_eq!(processor.validate("is_numeric", "abc"), Some(false));
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        assert_eq!(processor.transform("trim", "  hello  ".to_string()), Some("hello".to_string()));
        assert_eq!(processor.transform("uppercase", "test".to_string()), Some("TEST".to_string()));
        assert_eq!(processor.transform("reverse", "abc".to_string()), Some("cba".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let processor = create_default_processor();
        let result = processor.process_pipeline("  hello  ", &["not_empty"], &["trim", "uppercase"]);
        assert_eq!(result, Ok("HELLO".to_string()));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
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
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records.into_iter().map(|record| self.process(record)).collect()
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule(|record| {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationError("Empty values array".to_string()));
        }
        if record.timestamp < 0 {
            return Err(ProcessingError::ValidationError("Negative timestamp".to_string()));
        }
        Ok(())
    });

    processor.add_transformation(|mut record| {
        let sum: f64 = record.values.iter().sum();
        let avg = sum / record.values.len() as f64;
        record.metadata.insert("average".to_string(), avg.to_string());
        Ok(record)
    });

    processor.add_transformation(|mut record| {
        record.values = record.values.into_iter().map(|v| v.max(0.0)).collect();
        Ok(record)
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let processor = create_default_processor();
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0, -1.0],
            metadata,
        };

        let result = processor.process(record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.values, vec![1.0, 2.0, 3.0, 0.0]);
        assert!(processed.metadata.contains_key("average"));
    }

    #[test]
    fn test_validation_failure() {
        let processor = create_default_processor();
        
        let record = DataRecord {
            id: 2,
            timestamp: -100,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(result.is_err());
    }
}