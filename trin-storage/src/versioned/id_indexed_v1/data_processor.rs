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

    pub fn filter_valid_records(&self, records: Vec<Vec<String>>) -> Vec<Vec<String>> {
        records
            .into_iter()
            .filter(|record| self.validate_record(record))
            .collect()
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
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let invalid_record = vec!["".to_string(), "value".to_string()];
        
        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    pub valid_records: Vec<String>,
    pub invalid_records: Vec<String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            valid_records: Vec::new(),
            invalid_records: Vec::new(),
        }
    }

    pub fn process_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for (line_number, line) in reader.lines().enumerate() {
            let record = line?;
            
            if self.validate_record(&record) {
                self.valid_records.push(record);
            } else {
                self.invalid_records.push(format!("Line {}: {}", line_number + 1, record));
            }
        }

        Ok(())
    }

    fn validate_record(&self, record: &str) -> bool {
        let fields: Vec<&str> = record.split(',').collect();
        
        if fields.len() != 3 {
            return false;
        }

        fields.iter().all(|field| !field.trim().is_empty())
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        (self.valid_records.len(), self.invalid_records.len())
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
        writeln!(temp_file, "John,Doe,30").unwrap();
        writeln!(temp_file, "Jane,Smith,25").unwrap();
        writeln!(temp_file, "Invalid,Record").unwrap();
        writeln!(temp_file, ",,").unwrap();
        
        let result = processor.process_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let (valid, invalid) = processor.get_statistics();
        assert_eq!(valid, 2);
        assert_eq!(invalid, 2);
        assert_eq!(processor.valid_records.len(), 2);
        assert_eq!(processor.invalid_records.len(), 2);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: vec![
                ValidationRule {
                    min_value: 0.0,
                    max_value: 100.0,
                    required: true,
                },
            ],
        }
    }

    pub fn process_dataset(&mut self, dataset_id: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        for value in data {
            if !self.validate_value(*value) {
                return Err(format!("Value {} fails validation", value));
            }
        }

        let processed: Vec<f64> = data.iter().map(|&x| self.transform_value(x)).collect();
        self.cache.insert(dataset_id.to_string(), processed.clone());
        
        Ok(processed)
    }

    pub fn get_cached_data(&self, dataset_id: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_id)
    }

    fn validate_value(&self, value: f64) -> bool {
        self.validation_rules.iter().all(|rule| {
            if rule.required {
                value >= rule.min_value && value <= rule.max_value
            } else {
                true
            }
        })
    }

    fn transform_value(&self, value: f64) -> f64 {
        (value * 100.0).round() / 100.0
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_valid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![25.5, 50.0, 75.3];
        
        let result = processor.process_dataset("test1", &data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![25.5, 50.0, 75.3]);
    }

    #[test]
    fn test_process_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![150.0];
        
        let result = processor.process_dataset("test2", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        processor.process_dataset("cached", &data).unwrap();
        assert!(processor.get_cached_data("cached").is_some());
        
        processor.clear_cache();
        assert!(processor.get_cached_data("cached").is_none());
    }
}