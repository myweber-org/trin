
use std::error::Error;
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
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_number, line) in lines {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().any(|f| f.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid data records found".into());
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to validate".into());
        }

        let expected_len = records[0].len();
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", 
                    idx + 1, record.len(), expected_len).into());
            }
        }

        Ok(())
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<String>, Box<dyn Error>> {
        if column_index >= records[0].len() {
            return Err(format!("Column index {} out of bounds", column_index).into());
        }

        Ok(records
            .iter()
            .map(|record| record[column_index].clone())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records).is_ok());
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["John".to_string(), "30".to_string()],
            vec!["Alice".to_string(), "25".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 0).unwrap();
        
        assert_eq!(column, vec!["John", "Alice"]);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let max_value = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);

    (average, max_value, count)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
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
        
        processor.register_default_validators();
        processor.register_default_transformers();
        
        processor
    }
    
    fn register_default_validators(&mut self) {
        self.validators.insert(
            "email".to_string(),
            Box::new(|input: &str| {
                input.contains('@') && input.contains('.') && input.len() > 5
            })
        );
        
        self.validators.insert(
            "numeric".to_string(),
            Box::new(|input: &str| {
                input.parse::<f64>().is_ok()
            })
        );
        
        self.validators.insert(
            "not_empty".to_string(),
            Box::new(|input: &str| {
                !input.trim().is_empty()
            })
        );
    }
    
    fn register_default_transformers(&mut self) {
        self.transformers.insert(
            "uppercase".to_string(),
            Box::new(|input: String| {
                input.to_uppercase()
            })
        );
        
        self.transformers.insert(
            "trim".to_string(),
            Box::new(|input: String| {
                input.trim().to_string()
            })
        );
        
        self.transformers.insert(
            "reverse".to_string(),
            Box::new(|input: String| {
                input.chars().rev().collect()
            })
        );
    }
    
    pub fn validate(&self, validator_name: &str, input: &str) -> bool {
        match self.validators.get(validator_name) {
            Some(validator) => validator(input),
            None => false,
        }
    }
    
    pub fn transform(&self, transformer_name: &str, input: String) -> Option<String> {
        self.transformers.get(transformer_name)
            .map(|transformer| transformer(input))
    }
    
    pub fn process_pipeline(&self, input: String, operations: Vec<(&str, &str)>) -> Result<String, String> {
        let mut result = input;
        
        for (op_type, op_name) in operations {
            match op_type {
                "validate" => {
                    if !self.validate(op_name, &result) {
                        return Err(format!("Validation '{}' failed for input", op_name));
                    }
                }
                "transform" => {
                    result = match self.transform(op_name, result) {
                        Some(transformed) => transformed,
                        None => return Err(format!("Unknown transformer '{}'", op_name)),
                    };
                }
                _ => return Err(format!("Unknown operation type '{}'", op_type)),
            }
        }
        
        Ok(result)
    }
    
    pub fn register_validator(&mut self, name: String, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name, validator);
    }
    
    pub fn register_transformer(&mut self, name: String, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name, transformer);
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
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("numeric", "123.45"));
        assert!(!processor.validate("numeric", "abc"));
    }
    
    #[test]
    fn test_uppercase_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform("uppercase", "hello world".to_string());
        assert_eq!(result, Some("HELLO WORLD".to_string()));
    }
    
    #[test]
    fn test_processing_pipeline() {
        let processor = DataProcessor::new();
        let operations = vec![
            ("validate", "not_empty"),
            ("transform", "trim"),
            ("transform", "uppercase"),
        ];
        
        let result = processor.process_pipeline("  test data  ".to_string(), operations);
        assert_eq!(result, Ok("TEST DATA".to_string()));
    }
    
    #[test]
    fn test_custom_validator() {
        let mut processor = DataProcessor::new();
        processor.register_validator(
            "even_length".to_string(),
            Box::new(|input: &str| input.len() % 2 == 0)
        );
        
        assert!(processor.validate("even_length", "abcd"));
        assert!(!processor.validate("even_length", "abc"));
    }
}