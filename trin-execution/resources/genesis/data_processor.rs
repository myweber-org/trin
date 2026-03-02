
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            timestamp,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.timestamp.is_empty() {
            return Err("Timestamp cannot be empty".to_string());
        }
        Ok(())
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
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let timestamp = parts[3].to_string();
            
            let record = DataRecord::new(id, name, value, timestamp);
            if let Err(e) = record.validate() {
                eprintln!("Validation error on line {}: {}", index + 1, e);
                continue;
            }
            
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= threshold)
            .collect()
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records(&self) -> &Vec<DataRecord> {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, "2024-01-01".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,item1,10.5,2024-01-01").unwrap();
        writeln!(temp_file, "2,item2,20.3,2024-01-02").unwrap();
        writeln!(temp_file, "3,item3,15.7,2024-01-03").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let avg = processor.calculate_average();
        assert!((avg - 15.5).abs() < 0.1);
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().value, 20.3);
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
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
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
    valid_count: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
            valid_count: 0,
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

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
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let record = DataRecord::new(id, value, category);

            self.add_record(record);
        }

        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.total_value += record.get_value();
            self.valid_count += 1;
        }
        self.records.push(record);
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.valid_count > 0 {
            Some(self.total_value / self.valid_count as f64)
        } else {
            None
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_stats(&self) -> (usize, usize, Option<f64>) {
        (
            self.records.len(),
            self.valid_count,
            self.get_average_value(),
        )
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
        
        let record1 = DataRecord::new(1, 10.5, "A".to_string());
        let record2 = DataRecord::new(2, -5.0, "B".to_string());
        let record3 = DataRecord::new(3, 20.0, "A".to_string());
        
        processor.add_record(record1);
        processor.add_record(record2);
        processor.add_record(record3);
        
        let stats = processor.get_stats();
        assert_eq!(stats.0, 3);
        assert_eq!(stats.1, 2);
        assert_eq!(stats.2, Some(15.25));
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,15.0,TypeB").unwrap();
        writeln!(temp_file, "3,invalid,TypeC").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        
        let stats = processor.get_stats();
        assert_eq!(stats.0, 2);
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

    pub fn calculate_statistics(&self) -> HashMap<String, Stats> {
        let mut stats_map = HashMap::new();
        
        for (key, values) in &self.data {
            if values.is_empty() {
                continue;
            }

            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let median = if count as usize % 2 == 0 {
                let mid = count as usize / 2;
                (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
            } else {
                sorted_values[count as usize / 2]
            };

            stats_map.insert(key.clone(), Stats {
                mean,
                median,
                std_dev,
                min: *sorted_values.first().unwrap(),
                max: *sorted_values.last().unwrap(),
                count: values.len(),
            });
        }
        
        stats_map
    }

    pub fn transform_data<F>(&self, transform_fn: F) -> HashMap<String, Vec<f64>>
    where
        F: Fn(f64) -> f64,
    {
        self.data.iter()
            .map(|(key, values)| {
                let transformed: Vec<f64> = values.iter().map(|&x| transform_fn(x)).collect();
                (key.clone(), transformed)
            })
            .collect()
    }
}

pub struct Stats {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
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
    fn test_data_processor() {
        let rules = ValidationRules::new(
            0.0,
            100.0,
            vec!["temperature".to_string(), "humidity".to_string()]
        );
        
        let mut processor = DataProcessor::new(rules);
        
        assert!(processor.add_dataset(
            "temperature".to_string(),
            vec![20.5, 22.3, 19.8, 21.2]
        ).is_ok());
        
        assert!(processor.add_dataset(
            "pressure".to_string(),
            vec![1013.25]
        ).is_err());
        
        let stats = processor.calculate_statistics();
        assert!(stats.contains_key("temperature"));
        
        let transformed = processor.transform_data(|x| x * 1.8 + 32.0);
        assert_eq!(transformed.len(), 1);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid format at line {}", line_num + 1).into());
        }

        let id = parts[0].parse::<u32>()?;
        let value = parts[1].parse::<f64>()?;
        let category = parts[2].to_string();

        let record = DataRecord::new(id, value, category);
        if record.is_valid() {
            records.push(record);
        } else {
            eprintln!("Warning: Invalid record at line {}", line_num + 1);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>()
        / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, "alpha".to_string());
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record1 = DataRecord::new(0, 42.5, "alpha".to_string());
        assert!(!record1.is_valid());

        let record2 = DataRecord::new(1, -1.0, "alpha".to_string());
        assert!(!record2.is_valid());

        let record3 = DataRecord::new(1, 42.5, "".to_string());
        assert!(!record3.is_valid());
    }

    #[test]
    fn test_process_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,42.5,alpha")?;
        writuse std::error::Error;
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
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let invalid_record = vec!["".to_string(), "value".to_string()];
        
        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_extract_column() {
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&data, 1);
        
        assert_eq!(column, vec!["b".to_string(), "d".to_string()]);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
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
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub min_timestamp: i64,
    pub max_timestamp: i64,
    pub require_metadata: bool,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            max_values: 100,
            min_timestamp: 0,
            max_timestamp: i64::MAX,
            require_metadata: false,
        }
    }
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        Self { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationError(format!(
                "Too many values: {} > {}",
                record.values.len(),
                self.config.max_values
            )));
        }

        if record.timestamp < self.config.min_timestamp {
            return Err(ProcessingError::ValidationError(format!(
                "Timestamp too early: {} < {}",
                record.timestamp, self.config.min_timestamp
            )));
        }

        if record.timestamp > self.config.max_timestamp {
            return Err(ProcessingError::ValidationError(format!(
                "Timestamp too late: {} > {}",
                record.timestamp, self.config.max_timestamp
            )));
        }

        if self.config.require_metadata && record.metadata.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Metadata required but missing".to_string(),
            ));
        }

        Ok(())
    }

    pub fn transform_record(
        &self,
        record: DataRecord,
    ) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        let transformed_values: Vec<f64> = record
            .values
            .iter()
            .map(|&value| {
                if value.is_nan() || value.is_infinite() {
                    0.0
                } else {
                    value
                }
            })
            .collect();

        let mut transformed_metadata = record.metadata.clone();
        transformed_metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string(),
        );
        transformed_metadata.insert("value_count".to_string(), transformed_values.len().to_string());

        Ok(DataRecord {
            id: record.id,
            timestamp: record.timestamp,
            values: transformed_values,
            metadata: transformed_metadata,
        })
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        let mut errors = Vec::new();

        for record in records {
            match self.transform_record(record) {
                Ok(transformed) => results.push(transformed),
                Err(e) => errors.push(e.to_string()),
            }
        }

        if !errors.is_empty() {
            return Err(ProcessingError::ProcessingError(format!(
                "Batch processing completed with {} errors: {:?}",
                errors.len(),
                errors
            )));
        }

        Ok(results)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> Result<Statistics, ProcessingError> {
        if records.is_empty() {
            return Err(ProcessingError::InvalidData("No records provided".to_string()));
        }

        let mut total_values = 0;
        let mut sum = 0.0;
        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for record in records {
            self.validate_record(record)?;
            total_values += record.values.len();
            
            for &value in &record.values {
                if value.is_nan() || value.is_infinite() {
                    continue;
                }
                sum += value;
                min = min.min(value);
                max = max.max(value);
            }
        }

        let average = if total_values > 0 {
            sum / total_values as f64
        } else {
            0.0
        };

        Ok(Statistics {
            total_records: records.len(),
            total_values,
            average,
            min,
            max,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Statistics {
    pub total_records: usize,
    pub total_values: usize,
    pub average: f64,
    pub min: f64,
    pub max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: {
                let mut map = HashMap::new();
                map.insert("source".to_string(), "test".to_string());
                map
            },
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_too_many_values() {
        let config = ProcessingConfig {
            max_values: 3,
            ..Default::default()
        };
        let processor = DataProcessor::new(config);
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let record = create_test_record();
        let result = processor.transform_record(record);
        assert!(result.is_ok());
        
        let transformed = result.unwrap();
        assert!(transformed.metadata.contains_key("processed_timestamp"));
        assert!(transformed.metadata.contains_key("value_count"));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let records = vec![create_test_record()];
        let stats = processor.calculate_statistics(&records);
        assert!(stats.is_ok());
        
        let stats = stats.unwrap();
        assert_eq!(stats.total_records, 1);
        assert_eq!(stats.total_values, 5);
        assert_eq!(stats.average, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
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
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: i64) -> Self {
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
            
            let timestamp = match parts[3].parse::<i64>() {
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

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, 1234567890);
        assert!(valid_record.is_valid());
        
        let invalid_name = DataRecord::new(2, "".to_string(), 10.5, 1234567890);
        assert!(!invalid_name.is_valid());
        
        let invalid_value = DataRecord::new(3, "test".to_string(), -1.0, 1234567890);
        assert!(!invalid_value.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,item1,10.5,1234567890").unwrap();
        writeln!(temp_file, "2,item2,20.0,1234567891").unwrap();
        writeln!(temp_file, "3,item3,30.7,1234567892").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.get_records().len(), 3);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "a".to_string(), 10.0, 1000));
        processor.records.push(DataRecord::new(2, "b".to_string(), 20.0, 1001));
        processor.records.push(DataRecord::new(3, "c".to_string(), 30.0, 1002));
        
        let (mean, variance, std_dev) = processor.calculate_statistics();
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_value_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "a".to_string(), 10.0, 1000));
        processor.records.push(DataRecord::new(2, "b".to_string(), 20.0, 1001));
        processor.records.push(DataRecord::new(3, "c".to_string(), 30.0, 1002));
        processor.records.push(DataRecord::new(4, "d".to_string(), 40.0, 1003));
        
        let filtered = processor.filter_by_value(15.0, 35.0);
        
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.value >= 15.0 && r.value <= 35.0));
    }
}