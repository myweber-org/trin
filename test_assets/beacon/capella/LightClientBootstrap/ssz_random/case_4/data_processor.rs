
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut processor = DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        };

        processor.register_validator("email", |s| s.contains('@') && s.contains('.'));
        processor.register_validator("phone", |s| s.chars().all(|c| c.is_numeric()));
        
        processor.register_transformer("uppercase", |s| s.to_uppercase());
        processor.register_transformer("trim", |s| s.trim().to_string());

        processor
    }

    pub fn register_validator(&mut self, name: &str, validator: impl Fn(&str) -> bool + 'static) {
        self.validators.insert(name.to_string(), Box::new(validator));
    }

    pub fn register_transformer(&mut self, name: &str, transformer: impl Fn(String) -> String + 'static) {
        self.transformers.insert(name.to_string(), Box::new(transformer));
    }

    pub fn validate(&self, validator_name: &str, data: &str) -> bool {
        self.validators
            .get(validator_name)
            .map(|validator| validator(data))
            .unwrap_or(false)
    }

    pub fn transform(&self, transformer_name: &str, data: String) -> Option<String> {
        self.transformers
            .get(transformer_name)
            .map(|transformer| transformer(data))
    }

    pub fn process_pipeline(&self, data: &str, operations: Vec<(&str, &str)>) -> Result<String, String> {
        let mut result = data.to_string();

        for (op_type, op_name) in operations {
            match op_type {
                "validate" => {
                    if !self.validate(op_name, &result) {
                        return Err(format!("Validation '{}' failed for data: {}", op_name, result));
                    }
                }
                "transform" => {
                    result = self.transform(op_name, result)
                        .ok_or_else(|| format!("Unknown transformer: {}", op_name))?;
                }
                _ => return Err(format!("Unknown operation type: {}", op_type)),
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }

    #[test]
    fn test_uppercase_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform("uppercase", "hello".to_string());
        assert_eq!(result, Some("HELLO".to_string()));
    }

    #[test]
    fn test_processing_pipeline() {
        let processor = DataProcessor::new();
        let operations = vec![
            ("validate", "email"),
            ("transform", "uppercase"),
            ("transform", "trim"),
        ];
        
        let result = processor.process_pipeline("  test@example.com  ", operations);
        assert_eq!(result, Ok("TEST@EXAMPLE.COM".to_string()));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        let mut values = Vec::new();
        
        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert!(processor.validate_record(&records[0]));
        
        let stats = processor.calculate_statistics(&records, 1);
        assert!(stats.is_some());
        
        let (mean, _, _) = stats.unwrap();
        assert!((mean - 30.0).abs() < 0.001);
    }
}