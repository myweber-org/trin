
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::DuplicateTag => write!(f, "Tags must be unique"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let mut seen_tags = HashMap::new();
        for tag in &self.tags {
            if seen_tags.contains_key(tag) {
                return Err(ValidationError::DuplicateTag);
            }
            seen_tags.insert(tag.clone(), true);
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> &mut Self {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
        self.tags.sort();
        self.tags.dedup();
        self
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<Vec<DataRecord>, ValidationError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        let mut transformed = record.clone();
        transformed.transform(multiplier);
        processed.push(transformed);
    }
    
    Ok(processed)
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 10.5,
            tags: vec![],
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_record() {
        let mut record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 10.0,
            tags: vec!["b".to_string(), "a".to_string(), "b".to_string()],
        };
        
        record.transform(2.0);
        
        assert_eq!(record.name, "TEST");
        assert_eq!(record.value, 20.0);
        assert_eq!(record.tags, vec!["a".to_string(), "b".to_string()]);
    }
    
    #[test]
    fn test_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "A".to_string(),
                value: 10.0,
                tags: vec![],
            },
            DataRecord {
                id: 2,
                name: "B".to_string(),
                value: 20.0,
                tags: vec![],
            },
            DataRecord {
                id: 3,
                name: "C".to_string(),
                value: 30.0,
                tags: vec![],
            },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(self.records.len())
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

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value.is_finite() && !r.category.is_empty())
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.1);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let stats = processor.get_statistics();
        assert!((stats.0 - 10.5).abs() < 0.1);
        assert!((stats.1 - 20.3).abs() < 0.1);
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
pub enum DataError {
    InvalidId,
    InvalidName,
    InvalidValue,
    InvalidCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidName => write!(f, "Invalid record name"),
            DataError::InvalidValue => write!(f, "Invalid record value"),
            DataError::InvalidCategory => write!(f, "Invalid record category"),
            DataError::DuplicateRecord => write!(f, "Duplicate record found"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_totals: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_totals: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::InvalidName);
        }
        
        if record.value <= 0.0 {
            return Err(DataError::InvalidValue);
        }
        
        if record.category.trim().is_empty() {
            return Err(DataError::InvalidCategory);
        }
        
        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }
        
        self.records.insert(record.id, record.clone());
        
        let total = self.category_totals
            .entry(record.category.clone())
            .or_insert(0.0);
        *total += record.value;
        
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_total(&self, category: &str) -> f64 {
        self.category_totals.get(category).copied().unwrap_or(0.0)
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        if let Some(record) = self.records.remove(&id) {
            if let Some(total) = self.category_totals.get_mut(&record.category) {
                *total -= record.value;
                if *total <= 0.0 {
                    self.category_totals.remove(&record.category);
                }
            }
            Some(record)
        } else {
            None
        }
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn category_count(&self) -> usize {
        self.category_totals.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.category_totals.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: String::from("Test Record"),
            value: 100.0,
            category: String::from("Test"),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: String::from(""),
            value: -10.0,
            category: String::from(""),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_total() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: String::from("Record 1"),
            value: 50.0,
            category: String::from("CategoryA"),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: String::from("Record 2"),
            value: 75.0,
            category: String::from("CategoryA"),
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        assert_eq!(processor.get_category_total("CategoryA"), 125.0);
    }

    #[test]
    fn test_remove_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: String::from("Test Record"),
            value: 100.0,
            category: String::from("Test"),
        };
        
        processor.add_record(record).unwrap();
        assert_eq!(processor.record_count(), 1);
        
        let removed = processor.remove_record(1);
        assert!(removed.is_some());
        assert_eq!(processor.record_count(), 0);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: HashMap<String, ValidationRule>,
}

pub struct ValidationRule {
    min_value: Option<f64>,
    max_value: Option<f64>,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) {
        self.data.insert(key.to_string(), values);
    }

    pub fn set_validation_rule(&mut self, key: &str, rule: ValidationRule) {
        self.validation_rules.insert(key.to_string(), rule);
    }

    pub fn validate(&self) -> Result<(), String> {
        for (key, rule) in &self.validation_rules {
            if rule.required {
                if !self.data.contains_key(key) {
                    return Err(format!("Required dataset '{}' is missing", key));
                }
            }

            if let Some(values) = self.data.get(key) {
                for &value in values {
                    if let Some(min) = rule.min_value {
                        if value < min {
                            return Err(format!("Value {} in dataset '{}' is below minimum {}", value, key, min));
                        }
                    }
                    
                    if let Some(max) = rule.max_value {
                        if value > max {
                            return Err(format!("Value {} in dataset '{}' exceeds maximum {}", value, key, max));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn normalize(&mut self, key: &str) -> Result<Vec<f64>, String> {
        let values = self.data.get(key)
            .ok_or_else(|| format!("Dataset '{}' not found", key))?;
        
        if values.is_empty() {
            return Ok(Vec::new());
        }

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return Ok(vec![0.0; values.len()]);
        }

        let normalized: Vec<f64> = values.iter()
            .map(|&v| (v - min) / (max - min))
            .collect();

        Ok(normalized)
    }

    pub fn calculate_statistics(&self, key: &str) -> Result<Statistics, String> {
        let values = self.data.get(key)
            .ok_or_else(|| format!("Dataset '{}' not found", key))?;

        if values.is_empty() {
            return Err("Dataset is empty".to_string());
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        Ok(Statistics {
            count: values.len(),
            mean,
            variance,
            std_dev: variance.sqrt(),
        })
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl ValidationRule {
    pub fn new() -> Self {
        ValidationRule {
            min_value: None,
            max_value: None,
            required: false,
        }
    }

    pub fn with_min(mut self, min: f64) -> Self {
        self.min_value = Some(min);
        self
    }

    pub fn with_max(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperatures", vec![20.5, 22.1, 19.8, 25.3]);
        
        let rule = ValidationRule::new()
            .with_min(15.0)
            .with_max(30.0)
            .required();
        
        processor.set_validation_rule("temperatures", rule);
        
        assert!(processor.validate().is_ok());
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("scores", vec![10.0, 20.0, 30.0, 40.0]);
        
        let normalized = processor.normalize("scores").unwrap();
        assert_eq!(normalized, vec![0.0, 0.3333333333333333, 0.6666666666666666, 1.0]);
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values", vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        
        let stats = processor.calculate_statistics("values").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.variance, 2.0);
        assert_eq!(stats.std_dev, 2.0f64.sqrt());
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>], expected_columns: usize) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.get(column_index).cloned())
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
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["x".to_string(), "y".to_string()],
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records, 3);
        
        assert_eq!(invalid, vec![1]);
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["apple".to_string(), "red".to_string()],
            vec!["banana".to_string(), "yellow".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let colors = processor.extract_column(&records, 1);
        
        assert_eq!(colors, vec!["red", "yellow"]);
    }
}