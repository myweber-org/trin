
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field_name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub required: bool,
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

    pub fn process_dataset(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<HashMap<String, f64>>, String> {
        let mut processed = Vec::new();

        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(validated_record) => {
                    let transformed = self.transform_record(&validated_record);
                    processed.push(transformed);
                    self.cache_record(index, &transformed);
                }
                Err(e) => return Err(format!("Validation failed at record {}: {}", index, e)),
            }
        }

        Ok(processed)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<HashMap<String, f64>, String> {
        let mut validated = HashMap::new();

        for rule in &self.validation_rules {
            match record.get(&rule.field_name) {
                Some(&value) => {
                    if value < rule.min_value || value > rule.max_value {
                        return Err(format!(
                            "Field '{}' value {} out of range [{}, {}]",
                            rule.field_name, value, rule.min_value, rule.max_value
                        ));
                    }
                    validated.insert(rule.field_name.clone(), value);
                }
                None => {
                    if rule.required {
                        return Err(format!("Required field '{}' missing", rule.field_name));
                    }
                }
            }
        }

        Ok(validated)
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut transformed = HashMap::new();

        for (key, value) in record {
            let transformed_value = match key.as_str() {
                "temperature" => (value - 32.0) * 5.0 / 9.0,
                "pressure" => value * 1000.0,
                "humidity" => value.min(100.0).max(0.0),
                _ => *value,
            };
            transformed.insert(key.clone(), transformed_value);
        }

        transformed
    }

    fn cache_record(&mut self, index: usize, record: &HashMap<String, f64>) {
        for (key, value) in record {
            self.cache
                .entry(key.clone())
                .or_insert_with(Vec::new)
                .push(*value);
        }
    }

    pub fn get_cached_stats(&self, field: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(field).map(|values| {
            let count = values.len() as f64;
            let sum: f64 = values.iter().sum();
            let mean = sum / count;
            let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count;
            let std_dev = variance.sqrt();

            (mean, variance, std_dev)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 150.0,
            required: true,
        });

        let dataset = vec![
            [("temperature".to_string(), 68.0)].iter().cloned().collect(),
            [("temperature".to_string(), 32.0)].iter().cloned().collect(),
        ];

        let result = processor.process_dataset(&dataset);
        assert!(result.is_ok());
        
        if let Ok(processed) = result {
            assert_eq!(processed.len(), 2);
            assert!((processed[0]["temperature"] - 20.0).abs() < 0.001);
        }
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

pub struct DataProcessor {
    validation_rules: Vec<ValidationRule>,
    transformation_pipeline: Vec<Transformation>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformation_pipeline.push(transformation);
    }

    pub fn process(&self, record: &mut DataRecord) -> Result<(), DataError> {
        for rule in &self.validation_rules {
            rule.validate(record)?;
        }

        for transformation in &self.transformation_pipeline {
            transformation.apply(record);
        }

        Ok(())
    }

    pub fn batch_process(&self, records: &mut [DataRecord]) -> Vec<Result<(), DataError>> {
        records
            .iter_mut()
            .map(|record| self.process(record))
            .collect()
    }
}

pub trait ValidationRule {
    fn validate(&self, record: &DataRecord) -> Result<(), DataError>;
}

pub trait Transformation {
    fn apply(&self, record: &mut DataRecord);
}

pub struct RequiredFieldRule {
    field_name: String,
}

impl RequiredFieldRule {
    pub fn new(field_name: &str) -> Self {
        RequiredFieldRule {
            field_name: field_name.to_string(),
        }
    }
}

impl ValidationRule for RequiredFieldRule {
    fn validate(&self, record: &DataRecord) -> Result<(), DataError> {
        if !record.values.contains_key(&self.field_name) {
            return Err(DataError::MissingField(self.field_name.clone()));
        }
        Ok(())
    }
}

pub struct NormalizeTransformation {
    factor: f64,
}

impl NormalizeTransformation {
    pub fn new(factor: f64) -> Self {
        NormalizeTransformation { factor }
    }
}

impl Transformation for NormalizeTransformation {
    fn apply(&self, record: &mut DataRecord) {
        for value in record.values.values_mut() {
            *value /= self.factor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_field_validation() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(RequiredFieldRule::new("temperature"));

        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::new(),
            tags: vec![],
        };

        let result = processor.process(&mut record);
        assert!(result.is_err());

        record.values.insert("temperature".to_string(), 25.5);
        let result = processor.process(&mut record);
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalize_transformation() {
        let mut processor = DataProcessor::new();
        processor.add_transformation(NormalizeTransformation::new(10.0));

        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("value1".to_string(), 100.0),
                ("value2".to_string(), 50.0),
            ]),
            tags: vec![],
        };

        processor.process(&mut record).unwrap();
        assert_eq!(record.values.get("value1"), Some(&10.0));
        assert_eq!(record.values.get("value2"), Some(&5.0));
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

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
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

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64)> {
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
        let mean = sum / values.len() as f64;
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        Some((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.0"]);
    }

    #[test]
    fn test_record_validation() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "123".to_string()];
        let invalid_record = vec!["".to_string(), "123".to_string()];
        
        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(',', false);
        let records = vec![
            vec!["10.0".to_string(), "20.0".to_string()],
            vec!["20.0".to_string(), "30.0".to_string()],
            vec!["30.0".to_string(), "40.0".to_string()],
        ];
        
        let stats = processor.calculate_statistics(&records, 0);
        assert!(stats.is_some());
        
        let (mean, std_dev) = stats.unwrap();
        assert!((mean - 20.0).abs() < 0.001);
        assert!((std_dev - 8.164966).abs() < 0.001);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut values = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(num) = line.trim().parse::<f64>() {
                values.push(num);
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn calculate_std_dev(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.calculate_mean()?;
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.values.len() - 1) as f64;
        Some(variance.sqrt())
    }

    pub fn get_min_max(&self) -> Option<(f64, f64)> {
        if self.values.is_empty() {
            return None;
        }
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        Some((min, max))
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_dataset() {
        let ds = DataSet::new();
        assert_eq!(ds.count(), 0);
        assert_eq!(ds.calculate_mean(), None);
        assert_eq!(ds.calculate_std_dev(), None);
        assert_eq!(ds.get_min_max(), None);
    }

    #[test]
    fn test_basic_statistics() {
        let mut ds = DataSet::new();
        ds.add_value(10.0);
        ds.add_value(20.0);
        ds.add_value(30.0);
        
        assert_eq!(ds.count(), 3);
        assert_eq!(ds.calculate_mean(), Some(20.0));
        assert_eq!(ds.get_min_max(), Some((10.0, 30.0)));
    }

    #[test]
    fn test_csv_parsing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1.5\n2.5\n3.5\n4.5")?;
        
        let ds = DataSet::from_csv(temp_file.path().to_str().unwrap())?;
        assert_eq!(ds.count(), 4);
        assert_eq!(ds.calculate_mean(), Some(3.0));
        assert_eq!(ds.get_min_max(), Some((1.5, 4.5)));
        
        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut loaded_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    loaded_count += 1;
                }
                Err(e) => eprintln!("Warning: Skipping invalid record at line {}: {}", line_num + 1, e),
            }
        }

        Ok(loaded_count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.calculate_total_value() / self.records.len() as f64)
        }
    }

    pub fn find_max_value_record(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string()).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_record() {
        assert!(DataRecord::new(1, -5.0, "test".to_string()).is_err());
        assert!(DataRecord::new(1, 5.0, "".to_string()).is_err());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,100.0,category_a").unwrap();
        writeln!(temp_file, "2,200.0,category_b").unwrap();
        writeln!(temp_file, "3,300.0,category_a").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.records.len(), 3);
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 100.0, "alpha".to_string()).unwrap());
        processor.records.push(DataRecord::new(2, 200.0, "beta".to_string()).unwrap());
        processor.records.push(DataRecord::new(3, 300.0, "alpha".to_string()).unwrap());

        let filtered = processor.filter_by_category("alpha");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculations() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test".to_string()).unwrap());
        processor.records.push(DataRecord::new(2, 20.0, "test".to_string()).unwrap());
        processor.records.push(DataRecord::new(3, 30.0, "test".to_string()).unwrap());

        assert_eq!(processor.calculate_total_value(), 60.0);
        assert_eq!(processor.get_average_value(), Some(20.0));
        assert_eq!(processor.find_max_value_record().unwrap().id, 3);
    }
}use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    EmptyCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::EmptyCategory => write!(f, "Category cannot be empty"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        if category.trim().is_empty() {
            return Err(ValidationError::EmptyCategory);
        }

        Ok(Self {
            id,
            value,
            category: category.trim().to_string(),
        })
    }

    pub fn transform_value(&mut self, multiplier: f64) -> Result<(), ValidationError> {
        let new_value = self.value * multiplier;
        if new_value < 0.0 || new_value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        self.value = new_value;
        Ok(())
    }

    pub fn normalize(&self) -> f64 {
        self.value / 1000.0
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<f64>, ValidationError> {
    let mut results = Vec::with_capacity(records.len());
    
    for record in records {
        record.transform_value(1.5)?;
        results.push(record.normalize());
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, "sample".to_string());
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 500.0, "sample".to_string());
        assert!(matches!(record, Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, 200.0, "test".to_string()).unwrap();
        assert!(record.transform_value(2.0).is_ok());
        assert_eq!(record.value, 400.0);
    }

    #[test]
    fn test_normalize() {
        let record = DataRecord::new(1, 250.0, "data".to_string()).unwrap();
        assert_eq!(record.normalize(), 0.25);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;
        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records.iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats.0 - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        
        self.data.clear();
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            if let Ok(value) = line.parse::<f64>() {
                self.data.push(value);
            }
        }
        
        self.metadata.insert("source".to_string(), filepath.to_string());
        self.metadata.insert("loaded_at".to_string(), chrono::Local::now().to_rfc3339());
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }
        
        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.data.iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        
        stats
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&value| value >= threshold)
            .cloned()
            .collect()
    }

    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
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
        writeln!(temp_file, "value").unwrap();
        writeln!(temp_file, "10.5").unwrap();
        writeln!(temp_file, "20.3").unwrap();
        writeln!(temp_file, "15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats["mean"] - 15.5).abs() < 0.1);
        assert!((stats["std_dev"] - 4.9).abs() < 0.1);
        
        let filtered = processor.filter_data(15.0);
        assert_eq!(filtered.len(), 2);
        
        let metadata = processor.get_metadata();
        assert!(metadata.contains_key("source"));
    }
}