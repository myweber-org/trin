
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

    pub fn process(&self, data: &str) -> Result<String, String> {
        if !self.validate("email", data) {
            return Err("Invalid email format".to_string());
        }

        let transformed = self.transform("uppercase", data.to_string())
            .ok_or("Transformation failed")?;

        Ok(transformed)
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        let mut processor = DataProcessor::new();
        
        processor.add_validator("email", Box::new(|s| s.contains('@')));
        processor.add_transformer("uppercase", Box::new(|s| s.to_uppercase()));
        processor.add_transformer("trim", Box::new(|s| s.trim().to_string()));
        
        processor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::default();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }

    #[test]
    fn test_uppercase_transformation() {
        let processor = DataProcessor::default();
        let result = processor.transform("uppercase", "hello".to_string());
        assert_eq!(result, Some("HELLO".to_string()));
    }

    #[test]
    fn test_processing_pipeline() {
        let processor = DataProcessor::default();
        let result = processor.process("user@domain.com");
        assert_eq!(result, Ok("USER@DOMAIN.COM".to_string()));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        if let Some(header) = lines.next() {
            let headers: Vec<String> = header?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            for line in lines {
                let line = line?;
                let values: Vec<String> = line
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                
                let mut record = HashMap::new();
                for (i, header) in headers.iter().enumerate() {
                    if i < values.len() {
                        record.insert(header.clone(), values[i].clone());
                    }
                }
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn calculate_average(&self, column: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;
        
        for record in &self.records {
            if let Some(value) = record.get(column) {
                if let Ok(num) = value.parse::<f64>() {
                    sum += num;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }

    pub fn get_unique_values(&self, column: &str) -> Vec<String> {
        let mut unique_values = HashMap::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column) {
                unique_values.insert(value.clone(), ());
            }
        }
        
        unique_values.keys().cloned().collect()
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<HashMap<String, String>>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        self.records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
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
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,85.5").unwrap();
        writeln!(temp_file, "Bob,30,92.0").unwrap();
        writeln!(temp_file, "Charlie,25,78.5").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let avg_score = processor.calculate_average("score");
        assert!(avg_score.is_some());
        assert!((avg_score.unwrap() - 85.333).abs() < 0.001);
        
        let unique_ages = processor.get_unique_values("age");
        assert_eq!(unique_ages.len(), 2);
        
        let filtered = processor.filter_records(|record| {
            record.get("age").map_or(false, |age| age == "25")
        });
        assert_eq!(filtered.len(), 2);
    }
}