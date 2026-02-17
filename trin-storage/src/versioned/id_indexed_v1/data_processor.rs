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
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    InvalidCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.name.trim().is_empty() {
        return Err(ValidationError::EmptyName);
    }
    
    if record.value < 0.0 {
        return Err(ValidationError::NegativeValue);
    }
    
    let valid_categories = ["A", "B", "C"];
    if !valid_categories.contains(&record.category.as_str()) {
        return Err(ValidationError::InvalidCategory);
    }
    
    Ok(())
}

pub fn transform_records(records: Vec<DataRecord>) -> HashMap<String, Vec<DataRecord>> {
    let mut grouped = HashMap::new();
    
    for record in records {
        grouped
            .entry(record.category.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    grouped
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

pub fn process_data_batch(records: Vec<DataRecord>) -> Result<HashMap<String, (f64, f64, f64)>, ValidationError> {
    let mut results = HashMap::new();
    
    for record in &records {
        validate_record(record)?;
    }
    
    let grouped = transform_records(records);
    
    for (category, category_records) in grouped {
        let stats = calculate_statistics(&category_records);
        results.insert(category, stats);
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 20.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 30.0, category: "A".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
    
    #[test]
    fn test_process_data_batch() {
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 20.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 15.0, category: "B".to_string() },
        ];
        
        let result = process_data_batch(records).unwrap();
        
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("A"));
        assert!(result.contains_key("B"));
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: ValidationRules,
}

pub struct ValidationRules {
    min_value: f64,
    max_value: f64,
    required_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(rules: ValidationRules) -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if !self.validation_rules.required_keys.contains(&key) {
            return Err(format!("Key '{}' is not in required keys list", key));
        }

        for &value in &values {
            if value < self.validation_rules.min_value || value > self.validation_rules.max_value {
                return Err(format!("Value {} is outside allowed range [{}, {}]", 
                    value, self.validation_rules.min_value, self.validation_rules.max_value));
            }
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len() as f64;
            let sum: f64 = values.iter().sum();
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();

            Statistics {
                count: values.len(),
                mean,
                std_dev,
                min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
            }
        })
    }

    pub fn normalize_data(&mut self, key: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(key) {
            let stats = self.calculate_statistics(key).unwrap();
            
            if stats.std_dev == 0.0 {
                return Err("Cannot normalize data with zero standard deviation".to_string());
            }

            for value in values.iter_mut() {
                *value = (*value - stats.mean) / stats.std_dev;
            }
            Ok(())
        } else {
            Err(format!("Key '{}' not found in dataset", key))
        }
    }

    pub fn merge_datasets(&mut self, other: DataProcessor) {
        for (key, values) in other.data {
            self.data.entry(key)
                .and_modify(|existing| existing.extend_from_slice(&values))
                .or_insert(values);
        }
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    pub fn get_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.data.get(key)
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

impl ValidationRules {
    pub fn new(min_value: f64, max_value: f64, required_keys: Vec<String>) -> Self {
        ValidationRules {
            min_value,
            max_value,
            required_keys,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor_validation() {
        let rules = ValidationRules::new(
            0.0,
            100.0,
            vec!["temperature".to_string(), "humidity".to_string()]
        );
        
        let mut processor = DataProcessor::new(rules);
        
        assert!(processor.add_dataset("temperature".to_string(), vec![20.0, 25.0, 30.0]).is_ok());
        assert!(processor.add_dataset("pressure".to_string(), vec![1013.0]).is_err());
        assert!(processor.add_dataset("temperature".to_string(), vec![-10.0]).is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let rules = ValidationRules::new(f64::MIN, f64::MAX, vec!["test".to_string()]);
        let mut processor = DataProcessor::new(rules);
        
        processor.add_dataset("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        
        let stats = processor.calculate_statistics("test").unwrap();
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_data_normalization() {
        let rules = ValidationRules::new(f64::MIN, f64::MAX, vec!["values".to_string()]);
        let mut processor = DataProcessor::new(rules);
        
        processor.add_dataset("values".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        processor.normalize_data("values").unwrap();
        
        let normalized = processor.get_data("values").unwrap();
        let stats = processor.calculate_statistics("values").unwrap();
        
        assert!(stats.mean.abs() < 1e-10);
        assert!((stats.std_dev - 1.0).abs() < 1e-10);
    }
}