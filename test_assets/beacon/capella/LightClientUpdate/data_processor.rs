use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        self.records.clear();

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn save_filtered_results(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in filtered {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.records
            .iter()
            .map(|record| record.value)
            .fold(f64::NEG_INFINITY, f64::max);

        (count, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let test_data = vec![
            DataRecord { id: 1, name: "Item A".to_string(), value: 10.5, category: "Category1".to_string() },
            DataRecord { id: 2, name: "Item B".to_string(), value: 20.3, category: "Category2".to_string() },
            DataRecord { id: 3, name: "Item C".to_string(), value: 15.7, category: "Category1".to_string() },
        ];

        processor.records = test_data;

        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average();
        assert!((avg - 15.5).abs() < 0.1);

        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, &'static str> {
        if value < 0.0 {
            return Err("Value cannot be negative");
        }
        if category.trim().is_empty() {
            return Err("Category cannot be empty");
        }
        
        Ok(DataRecord {
            id,
            value,
            category: category.to_string(),
        })
    }
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
        let reader = BufReader::new(file);
        let mut count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };
            
            let category = parts[2].trim();
            
            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(_) => continue,
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
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
    
    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
    
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }
    
    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);
        assert!(processor.calculate_average().is_none());
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
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

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        let validated_data = self.validate_data(data)?;
        let transformed_data = self.apply_transformations(&validated_data);
        
        self.cache.insert(dataset_name.to_string(), transformed_data.clone());
        
        Ok(transformed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        for rule in &self.validation_rules {
            if rule.required && data.is_empty() {
                return Err(format!("Field '{}' is required", rule.field_name));
            }
            
            for &value in data {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!(
                        "Value {} for field '{}' is outside allowed range [{}, {}]",
                        value, rule.field_name, rule.min_value, rule.max_value
                    ));
                }
            }
        }
        
        Ok(data.to_vec())
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        let mean = self.calculate_mean(data);
        let std_dev = self.calculate_std_dev(data, mean);
        
        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn calculate_mean(&self, data: &[f64]) -> f64 {
        let sum: f64 = data.iter().sum();
        sum / data.len() as f64
    }

    fn calculate_std_dev(&self, data: &[f64], mean: f64) -> f64 {
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        variance.sqrt()
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("temperature", -50.0, 150.0, true);
        processor.add_validation_rule(rule);

        let data = vec![20.5, 25.0, 30.2, 18.7];
        let result = processor.process_dataset("weather", &data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("weather").unwrap().len(), 4);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("pressure", 0.0, 100.0, true);
        processor.add_validation_rule(rule);

        let invalid_data = vec![95.0, 101.5, 85.0];
        let result = processor.process_dataset("pressure_data", &invalid_data);
        
        assert!(result.is_err());
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
    #[error("Value out of range: {0}")]
    OutOfRange(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidFormat);
        }
        
        if self.timestamp < 0 {
            return Err(DataError::OutOfRange("timestamp".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::MissingField("values".to_string()));
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) {
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
    }
    
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

pub fn process_records(records: &mut [DataRecord], multiplier: f64) -> Result<(), DataError> {
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(multiplier);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: HashMap::from([("temperature".to_string(), 25.5)]),
            tags: vec!["sensor".to_string()],
        };
        
        assert!(record.validate().is_ok());
        
        record.id = 0;
        assert!(matches!(record.validate(), Err(DataError::InvalidFormat)));
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: HashMap::from([("pressure".to_string(), 100.0)]),
            tags: vec![],
        };
        
        record.transform(2.5);
        assert_eq!(*record.values.get("pressure").unwrap(), 250.0);
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.records() {
            let record = result?;
            if record.len() >= 3 {
                let id: u32 = record[0].parse()?;
                let value: f64 = record[1].parse()?;
                let category = record[2].to_string();

                self.records.push(DataRecord {
                    id,
                    value,
                    category,
                });
            }
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
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
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,45.5,TypeA").unwrap();
        writeln!(temp_file, "2,67.8,TypeB").unwrap();
        writeln!(temp_file, "3,89.2,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let type_a_records = processor.filter_by_category("TypeA");
        assert_eq!(type_a_records.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 3);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: DataRecord = result?;
            if record.is_valid() {
                self.records.push(record);
            }
        }
        Ok(())
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = Writer::from_path(path)?;
        for record in &self.records {
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "B".to_string(),
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        let test_record = DataRecord {
            id: 1,
            name: "Sample".to_string(),
            value: 25.0,
            category: "Test".to_string(),
        };
        processor.add_record(test_record.clone());
        processor.save_to_csv(path)?;

        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path)?;
        assert_eq!(new_processor.get_records().len(), 1);

        Ok(())
    }

    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord {
            id: 1,
            name: "Item1".to_string(),
            value: 10.0,
            category: "CatA".to_string(),
        });
        processor.add_record(DataRecord {
            id: 2,
            name: "Item2".to_string(),
            value: 20.0,
            category: "CatA".to_string(),
        });
        processor.add_record(DataRecord {
            id: 3,
            name: "Item3".to_string(),
            value: 30.0,
            category: "CatB".to_string(),
        });

        let filtered = processor.filter_by_category("CatA");
        assert_eq!(filtered.len(), 2);
        assert_eq!(processor.calculate_average(), 20.0);
    }
}
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

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        if records.is_empty() || column_index >= records[0].len() {
            return None;
        }

        let mut values = Vec::new();
        for record in records {
            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
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
        assert_eq!(records[0], vec!["Alice", "30", "50000.0"]);

        let stats = processor.calculate_statistics(&records, 1).unwrap();
        assert!((stats.0 - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_invalid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "Alice,30").unwrap();
        writeln!(temp_file, ",25").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
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

        let mut count = 0;
        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.validate_record(&record)?;
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), String> {
        if record.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if record.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if record.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> Statistics {
        let count = self.records.len();
        let avg = self.calculate_average().unwrap_or(0.0);
        let max = self.records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
        let min = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);

        Statistics {
            count,
            average: avg,
            maximum: max,
            minimum: min,
        }
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub count: usize,
    pub average: f64,
    pub maximum: f64,
    pub minimum: f64,
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,Category1").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);

        let category1_items = processor.filter_by_category("Category1");
        assert_eq!(category1_items.len(), 2);

        let stats = processor.get_statistics();
        assert_eq!(stats.count, 3);
        assert!((stats.average - 15.5).abs() < 0.1);
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: DataRecord = result?;
        record.validate()?;
        records.push(record);
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, -5.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord::new(1, 10.0, "A".to_string()),
            DataRecord::new(2, 20.0, "B".to_string()),
            DataRecord::new(3, 30.0, "C".to_string()),
        ];

        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

pub struct DataProcessor;

impl DataProcessor {
    pub fn load_from_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
        let mut reader = Reader::from_path(path)?;
        let mut records = Vec::new();

        for result in reader.deserialize() {
            let record: Record = result?;
            record.validate()?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn save_to_csv<P: AsRef<Path>>(records: &[Record], path: P) -> Result<(), Box<dyn Error>> {
        let mut writer = Writer::from_path(path)?;

        for record in records {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn filter_active(records: &[Record]) -> Vec<&Record> {
        records.iter().filter(|r| r.active).collect()
    }

    pub fn calculate_total(records: &[Record]) -> f64 {
        records.iter().map(|r| r.value).sum()
    }

    pub fn find_max_value(records: &[Record]) -> Option<&Record> {
        records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record::new(2, "".to_string(), -50.0, false);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let records = vec![
            Record::new(1, "Alice".to_string(), 150.0, true),
            Record::new(2, "Bob".to_string(), 200.0, false),
            Record::new(3, "Charlie".to_string(), 175.0, true),
        ];

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        DataProcessor::save_to_csv(&records, path)?;
        let loaded_records = DataProcessor::load_from_csv(path)?;

        assert_eq!(loaded_records.len(), 3);
        assert_eq!(DataProcessor::calculate_total(&loaded_records), 525.0);
        assert_eq!(DataProcessor::filter_active(&loaded_records).len(), 2);

        let max_record = DataProcessor::find_max_value(&loaded_records).unwrap();
        assert_eq!(max_record.id, 2);

        Ok(())
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub fn parse_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let name = parts[1].trim().to_string();
        let value = parts[2].parse::<f64>()?;
        let active = parts[3].parse::<bool>()?;

        records.push(Record::new(id, name, value, active));
    }

    Ok(records)
}

pub fn filter_valid_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.is_valid()).collect()
}

pub fn calculate_total_value(records: &[&Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 42.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -10.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            Record::new(1, "Alpha".to_string(), 100.0, true),
            Record::new(2, "".to_string(), 50.0, false),
            Record::new(3, "Beta".to_string(), -20.0, true),
        ];

        let valid = filter_valid_records(&records);
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0].id, 1);
    }

    #[test]
    fn test_total_calculation() {
        let records = vec![
            Record::new(1, "A".to_string(), 10.0, true),
            Record::new(2, "B".to_string(), 20.0, true),
            Record::new(3, "C".to_string(), 30.0, false),
        ];

        let valid = filter_valid_records(&records);
        let total = calculate_total_value(&valid);
        assert_eq!(total, 60.0);
    }
}