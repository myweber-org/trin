
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
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
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
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
        let record = DataRecord::new(1, 500.0, "test".to_string());
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, 500.0, "test".to_string());
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
        let record = DataRecord::new(1, 250.0, "test".to_string()).unwrap();
        assert_eq!(record.normalize(), 0.25);
    }
}
use std::error::Error;
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

        if let Some(header_result) = lines.next() {
            let header_line = header_result?;
            let headers: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();

            for line_result in lines {
                let line = line_result?;
                let values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
                
                if values.len() == headers.len() {
                    let mut record = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        record.insert(header.clone(), values[i].clone());
                    }
                    self.records.push(record);
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_average(&self, column_name: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value_str) = record.get(column_name) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
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

    pub fn count_unique_values(&self, column_name: &str) -> usize {
        let mut unique_values = std::collections::HashSet::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column_name) {
                unique_values.insert(value.clone());
            }
        }
        
        unique_values.len()
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

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_column_names(&self) -> Vec<String> {
        if let Some(first_record) = self.records.first() {
            first_record.keys().cloned().collect()
        } else {
            Vec::new()
        }
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
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let avg_age = processor.calculate_average("age");
        assert!(avg_age.is_some());
        assert!((avg_age.unwrap() - 30.0).abs() < 0.001);
        
        let unique_names = processor.count_unique_values("name");
        assert_eq!(unique_names, 3);
        
        let filtered = processor.filter_records(|record| {
            record.get("age").and_then(|a| a.parse::<i32>().ok()).unwrap_or(0) > 30
        });
        assert_eq!(filtered.len(), 1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: u64) -> Self {
        Self {
            id,
            name,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && self.timestamp > 0
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
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
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let timestamp = match parts[3].parse::<u64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, name, value, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, min_value: f64, max_value: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= min_value && r.value <= max_value)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&DataRecord> {
        self.records.iter().find(|r| r.name == name)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, 1234567890);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, 0);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,item1,10.5,1000").unwrap();
        writeln!(temp_file, "2,item2,20.0,2000").unwrap();
        writeln!(temp_file, "3,item3,30.5,3000").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.record_count(), 3);

        let filtered = processor.filter_by_value(15.0, 25.0);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "item2");

        let average = processor.calculate_average().unwrap();
        assert!((average - 20.3333).abs() < 0.0001);

        let found = processor.find_by_name("item1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 1);
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

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
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

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
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

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value < 0.0 || record.name.is_empty())
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, usize) {
        let total = self.records.len();
        let average = self.calculate_average();
        let invalid = self.validate_records().len();

        (total, average, invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.0,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,15.75,Category1").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 15.416).abs() < 0.001);

        let (total, _, invalid) = processor.get_statistics();
        assert_eq!(total, 3);
        assert_eq!(invalid, 0);
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
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    config: ProcessingConfig,
    statistics: ProcessingStats,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_value: f64,
    pub min_value: f64,
    pub allowed_tags: Vec<String>,
    pub enable_normalization: bool,
}

#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub records_processed: u64,
    pub validation_errors: u64,
    pub transformation_errors: u64,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor {
            config,
            statistics: ProcessingStats::default(),
        }
    }

    pub fn process_record(&mut self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.statistics.records_processed += 1;

        self.validate_record(record)?;
        let transformed = self.transform_record(record)?;
        
        Ok(transformed)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < self.config.min_value || record.value > self.config.max_value {
            return Err(ProcessingError::ValidationError(
                format!("Value {} out of range [{}, {}]", 
                    record.value, self.config.min_value, self.config.max_value)
            ));
        }

        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string()
            ));
        }

        for tag in &record.tags {
            if !self.config.allowed_tags.contains(tag) {
                return Err(ProcessingError::ValidationError(
                    format!("Tag '{}' is not allowed", tag)
                ));
            }
        }

        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        if self.config.enable_normalization {
            transformed.value = self.normalize_value(record.value)?;
        }

        transformed.name = transformed.name.to_uppercase();
        transformed.tags.sort();
        transformed.tags.dedup();

        Ok(transformed)
    }

    fn normalize_value(&self, value: f64) -> Result<f64, ProcessingError> {
        let range = self.config.max_value - self.config.min_value;
        if range <= 0.0 {
            return Err(ProcessingError::TransformationError(
                "Invalid range for normalization".to_string()
            ));
        }

        let normalized = (value - self.config.min_value) / range;
        
        if normalized.is_nan() || normalized.is_infinite() {
            return Err(ProcessingError::TransformationError(
                "Normalization produced invalid result".to_string()
            ));
        }

        Ok(normalized)
    }

    pub fn batch_process(
        &mut self, 
        records: Vec<DataRecord>
    ) -> (Vec<DataRecord>, Vec<ProcessingError>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.process_record(&record) {
                Ok(transformed) => processed.push(transformed),
                Err(e) => {
                    match e {
                        ProcessingError::ValidationError(_) => {
                            self.statistics.validation_errors += 1;
                        }
                        ProcessingError::TransformationError(_) => {
                            self.statistics.transformation_errors += 1;
                        }
                        _ => {}
                    }
                    errors.push(e);
                }
            }
        }

        (processed, errors)
    }

    pub fn get_statistics(&self) -> &ProcessingStats {
        &self.statistics
    }

    pub fn generate_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        
        summary.insert(
            "total_records".to_string(),
            self.statistics.records_processed.to_string()
        );
        summary.insert(
            "validation_errors".to_string(),
            self.statistics.validation_errors.to_string()
        );
        summary.insert(
            "transformation_errors".to_string(),
            self.statistics.transformation_errors.to_string()
        );
        
        let success_rate = if self.statistics.records_processed > 0 {
            let errors = self.statistics.validation_errors + self.statistics.transformation_errors;
            let success = self.statistics.records_processed - errors as u64;
            (success as f64 / self.statistics.records_processed as f64 * 100.0).round()
        } else {
            0.0
        };
        
        summary.insert("success_rate".to_string(), format!("{:.1}%", success_rate));
        
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_value: 100.0,
            min_value: 0.0,
            allowed_tags: vec!["important".to_string(), "normal".to_string(), "low".to_string()],
            enable_normalization: true,
        }
    }

    #[test]
    fn test_valid_record_processing() {
        let config = create_test_config();
        let mut processor = DataProcessor::new(config);
        
        let record = DataRecord {
            id: 1,
            name: "test record".to_string(),
            value: 50.0,
            tags: vec!["important".to_string()],
        };

        let result = processor.process_record(&record);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.name, "TEST RECORD");
        assert_eq!(processed.tags, vec!["important"]);
    }

    #[test]
    fn test_invalid_value_validation() {
        let config = create_test_config();
        let mut processor = DataProcessor::new(config);
        
        let record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 150.0,
            tags: vec![],
        };

        let result = processor.process_record(&record);
        assert!(result.is_err());
        
        if let Err(ProcessingError::ValidationError(msg)) = result {
            assert!(msg.contains("out of range"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_batch_processing() {
        let config = create_test_config();
        let mut processor = DataProcessor::new(config);
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "record1".to_string(),
                value: 25.0,
                tags: vec!["normal".to_string()],
            },
            DataRecord {
                id: 2,
                name: "record2".to_string(),
                value: 75.0,
                tags: vec!["important".to_string()],
            },
            DataRecord {
                id: 3,
                name: "".to_string(),
                value: 50.0,
                tags: vec![],
            },
        ];

        let (processed, errors) = processor.batch_process(records);
        
        assert_eq!(processed.len(), 2);
        assert_eq!(errors.len(), 1);
        assert_eq!(processor.get_statistics().records_processed, 3);
        assert_eq!(processor.get_statistics().validation_errors, 1);
    }

    #[test]
    fn test_normalization() {
        let config = ProcessingConfig {
            max_value: 200.0,
            min_value: 0.0,
            allowed_tags: vec![],
            enable_normalization: true,
        };
        
        let mut processor = DataProcessor::new(config);
        
        let record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 100.0,
            tags: vec![],
        };

        let result = processor.process_record(&record).unwrap();
        assert_eq!(result.value, 0.5);
    }
}