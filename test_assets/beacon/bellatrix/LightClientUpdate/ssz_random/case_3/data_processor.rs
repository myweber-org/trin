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
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String], expected_fields: usize) -> bool {
        record.len() == expected_fields && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index))
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_record_validation() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let invalid_record = vec!["".to_string(), "value".to_string()];
        
        assert!(processor.validate_record(&valid_record, 2));
        assert!(!processor.validate_record(&invalid_record, 2));
    }

    #[test]
    fn test_column_extraction() {
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&data, 0);
        
        assert_eq!(column, vec!["a".to_string(), "c".to_string()]);
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

    pub fn validate_numeric_fields(&self, data: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, String> {
        let mut numeric_values = Vec::new();
        
        for (row_num, row) in data.iter().enumerate() {
            if column_index >= row.len() {
                return Err(format!("Row {}: Column index out of bounds", row_num + 1));
            }
            
            match row[column_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Row {}: Invalid numeric value '{}'", 
                    row_num + 1, row[column_index])),
            }
        }
        
        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
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
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000.5"]);
    }

    #[test]
    fn test_numeric_validation() {
        let data = vec![
            vec!["10.5".to_string(), "text".to_string()],
            vec!["20.0".to_string(), "more".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let result = processor.validate_numeric_fields(&data, 0).unwrap();
        
        assert_eq!(result, vec![10.5, 20.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let processor = DataProcessor::new(',', false);
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert!((mean - 3.0).abs() < 0.0001);
        assert!((variance - 2.0).abs() < 0.0001);
        assert!((std_dev - 1.41421356).abs() < 0.0001);
    }
}
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

    pub fn process_data(&mut self, dataset_name: String, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Validation failed for {}: NaN values found", rule.field_name));
            }

            if let Some(&value) = data.iter().find(|&&x| x < rule.min_value || x > rule.max_value) {
                return Err(format!(
                    "Value {} out of range [{}, {}] for field {}",
                    value, rule.min_value, rule.max_value, rule.field_name
                ));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&x| x * 2.0)
            .collect();

        self.cache.insert(dataset_name, processed_data.clone());

        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
            let std_dev = variance.sqrt();

            Statistics {
                mean,
                variance,
                std_dev,
                count: data.len(),
                min: *data.iter().fold(&f64::INFINITY, |a, b| a.min(b)),
                max: *data.iter().fold(&f64::NEG_INFINITY, |a, b| a.max(b)),
            }
        })
    }
}

pub struct Statistics {
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule {
            field_name: "temperature".to_string(),
            min_value: -50.0,
            max_value: 100.0,
            required: true,
        };
        processor.add_validation_rule(rule);

        let data = vec![20.5, 25.0, 30.2, 15.8];
        let result = processor.process_data("test_data".to_string(), data);

        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("test_data").unwrap().len(), 4);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule {
            field_name: "pressure".to_string(),
            min_value: 0.0,
            max_value: 10.0,
            required: true,
        };
        processor.add_validation_rule(rule);

        let data = vec![5.0, 15.0, 3.0];
        let result = processor.process_data("invalid_data".to_string(), data);

        assert!(result.is_err());
    }
}
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

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            for part in parts {
                if let Ok(value) = part.trim().parse::<f64>() {
                    self.data.push(value);
                } else {
                    self.frequency_map
                        .entry(part.trim().to_string())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.data.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.data
            .iter()
            .map(|value| {
                let diff = mean - *value;
                diff * diff
            })
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn get_top_categories(&self, limit: usize) -> Vec<(String, u32)> {
        let mut entries: Vec<_> = self.frequency_map.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        
        entries
            .iter()
            .take(limit)
            .map(|(key, value)| (key.clone(), *value))
            .collect()
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data
            .iter()
            .filter(|&&value| value >= threshold)
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
        writeln!(temp_file, "10.5,20.3,15.7").unwrap();
        writeln!(temp_file, "category_a,25.1,category_b").unwrap();
        writeln!(temp_file, "category_a,18.9,30.2").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let stats = processor.calculate_statistics();
        assert!(stats.0 > 0.0);
        
        let top_categories = processor.get_top_categories(2);
        assert!(!top_categories.is_empty());
        
        let filtered = processor.filter_data(20.0);
        assert!(!filtered.is_empty());
    }
}use csv::Reader;
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
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), Box<dyn Error>> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if record.value < 0.0 {
        return Err("Value cannot be negative".into());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Invalid category".into());
    }
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
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
        
        if record.value <= 0.0 || record.value.is_nan() || record.value.is_infinite() {
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

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
        
        self.recalculate_category_totals();
    }

    fn recalculate_category_totals(&mut self) {
        self.category_totals.clear();
        
        for record in self.records.values() {
            let total = self.category_totals
                .entry(record.category.clone())
                .or_insert(0.0);
            *total += record.value;
        }
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
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
            name: String::from("Invalid"),
            value: 50.0,
            category: String::from("Test"),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_totals() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: String::from("Record 1"),
            value: 100.0,
            category: String::from("CategoryA"),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: String::from("Record 2"),
            value: 200.0,
            category: String::from("CategoryA"),
        };
        
        let record3 = DataRecord {
            id: 3,
            name: String::from("Record 3"),
            value: 150.0,
            category: String::from("CategoryB"),
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        processor.add_record(record3).unwrap();
        
        assert_eq!(processor.get_category_total("CategoryA"), 300.0);
        assert_eq!(processor.get_category_total("CategoryB"), 150.0);
        assert_eq!(processor.get_category_total("NonExistent"), 0.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: String::from("Test"),
            value: 100.0,
            category: String::from("Test"),
        };
        
        processor.add_record(record).unwrap();
        
        processor.transform_values(|v| v * 2.0);
        
        let updated_record = processor.get_record(1).unwrap();
        assert_eq!(updated_record.value, 200.0);
        assert_eq!(processor.get_category_total("Test"), 200.0);
    }
}