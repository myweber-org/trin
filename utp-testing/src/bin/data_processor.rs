use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }
        Ok(data.to_vec())
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).ln_1p().tanh())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_validation() {
        let processor = DataProcessor::new();
        let valid_data = vec![1.0, 2.0, 3.0];
        let invalid_data = vec![1.0, f64::NAN, 3.0];

        assert!(processor.validate_data(&valid_data).is_ok());
        assert!(processor.validate_data(&invalid_data).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        let normalized = processor.normalize_data(&data);

        let mean = normalized.iter().sum::<f64>() / normalized.len() as f64;
        assert!(mean.abs() < 1e-10);
    }

    #[test]
    fn test_caching() {
        let mut processor = DataProcessor::new();
        let data = vec![1.5, 2.5, 3.5];

        let result1 = processor.process_dataset("test", &data).unwrap();
        let result2 = processor.process_dataset("test", &data).unwrap();

        assert_eq!(result1, result2);
        assert_eq!(processor.cache_stats().0, 1);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        DataRecord {
            id,
            name,
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationFailed("ID cannot be zero".to_string()));
        }
        
        if self.name.trim().is_empty() {
            return Err(ProcessingError::ValidationFailed("Name cannot be empty".to_string()));
        }
        
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(ProcessingError::ValidationFailed("Value must be a valid number".to_string()));
        }
        
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::TransformationError(
                "Multiplier must be positive".to_string()
            ));
        }
        
        self.value *= multiplier;
        Ok(())
    }

    pub fn get_normalized_value(&self, min: f64, max: f64) -> Result<f64, ProcessingError> {
        if min >= max {
            return Err(ProcessingError::InvalidData(
                "Minimum must be less than maximum".to_string()
            ));
        }
        
        let normalized = (self.value - min) / (max - min);
        if normalized < 0.0 || normalized > 1.0 {
            Err(ProcessingError::InvalidData(
                "Value outside normalization range".to_string()
            ))
        } else {
            Ok(normalized)
        }
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::new();
    
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(2.0)?;
        
        if let Ok(normalized) = record.get_normalized_value(0.0, 100.0) {
            record.add_metadata("normalized_value".to_string(), normalized.to_string());
        }
        
        processed.push(record.clone());
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 50.0);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, "".to_string(), f64::NAN);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 10.0);
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 20.0);
        
        assert!(record.transform(0.0).is_err());
    }

    #[test]
    fn test_normalization() {
        let record = DataRecord::new(1, "Test".to_string(), 50.0);
        let normalized = record.get_normalized_value(0.0, 100.0);
        assert!(normalized.is_ok());
        assert_eq!(normalized.unwrap(), 0.5);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessedData {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub is_valid: bool,
}

#[derive(Debug)]
pub enum DataError {
    InvalidValue(f64),
    EmptyCategory,
    InvalidId,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidValue(val) => write!(f, "Invalid value: {}", val),
            DataError::EmptyCategory => write!(f, "Category cannot be empty"),
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_data(&self, id: u32, value: f64, category: &str) -> Result<(), DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }

        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue(value));
        }

        if category.trim().is_empty() {
            return Err(DataError::EmptyCategory);
        }

        Ok(())
    }

    pub fn process(&self, id: u32, value: f64, category: &str) -> Result<ProcessedData, DataError> {
        self.validate_data(id, value, category)?;

        let normalized_value = if value > self.threshold {
            value / 2.0
        } else {
            value * 1.5
        };

        let is_valid = normalized_value >= 50.0 && normalized_value <= 800.0;

        Ok(ProcessedData {
            id,
            value: normalized_value,
            category: category.to_string(),
            is_valid,
        })
    }

    pub fn batch_process(
        &self,
        items: &[(u32, f64, &str)],
    ) -> Vec<Result<ProcessedData, DataError>> {
        items
            .iter()
            .map(|&(id, value, category)| self.process(id, value, category))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_data_processing() {
        let processor = DataProcessor::new(500.0);
        let result = processor.process(1, 300.0, "electronics");

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.id, 1);
        assert_eq!(data.value, 450.0);
        assert_eq!(data.category, "electronics");
        assert!(data.is_valid);
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new(500.0);
        let result = processor.process(0, 300.0, "test");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DataError::InvalidId));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(500.0);
        let items = vec![
            (1, 300.0, "category_a"),
            (2, 600.0, "category_b"),
            (0, 200.0, "category_c"),
        ];

        let results = processor.batch_process(&items);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
    }
}
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use log::{error, info, warn};

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: &str) -> Self {
        DataRecord {
            id,
            value,
            timestamp: timestamp.to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid ID: cannot be zero".to_string());
        }
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(format!("Value {} out of valid range [0, 1000]", self.value));
        }
        if self.timestamp.is_empty() {
            return Err("Timestamp cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn to_csv(&self) -> String {
        format!("{},{},{}\n", self.id, self.value, self.timestamp)
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    processed_count: usize,
    error_count: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            processed_count: 0,
            error_count: 0,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        info!("Loading data from: {}", path.as_ref().display());
        
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                warn!("Invalid format at line {}: {}", line_num + 1, line);
                self.error_count += 1;
                continue;
            }
            
            match self.parse_record(&parts) {
                Ok(record) => {
                    self.records.push(record);
                    self.processed_count += 1;
                }
                Err(e) => {
                    error!("Failed to parse line {}: {}", line_num + 1, e);
                    self.error_count += 1;
                }
            }
        }
        
        info!("Loaded {} records, {} errors", self.processed_count, self.error_count);
        Ok(())
    }

    fn parse_record(&self, parts: &[&str]) -> Result<DataRecord, String> {
        let id = parts[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID: {}", e))?;
        
        let value = parts[1].parse::<f64>()
            .map_err(|e| format!("Invalid value: {}", e))?;
        
        let timestamp = parts[2].trim().to_string();
        
        let record = DataRecord::new(id, value, &timestamp);
        record.validate()?;
        
        Ok(record)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records.iter()
            .filter(|record| record.value >= threshold)
            .collect()
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

    pub fn save_results<P: AsRef<Path>>(&self, path: P, threshold: f64) -> io::Result<()> {
        let filtered = self.filter_by_threshold(threshold);
        let (mean, variance, std_dev) = self.calculate_statistics();
        
        let mut file = File::create(path)?;
        
        writeln!(file, "# Data Processing Results")?;
        writeln!(file, "# Total records: {}", self.records.len())?;
        writeln!(file, "# Processed: {}, Errors: {}", self.processed_count, self.error_count)?;
        writeln!(file, "# Statistics - Mean: {:.2}, Variance: {:.2}, StdDev: {:.2}", 
                 mean, variance, std_dev)?;
        writeln!(file, "# Threshold filter: {}", threshold)?;
        writeln!(file, "# Filtered records: {}", filtered.len())?;
        writeln!(file, "# ID,Value,Timestamp")?;
        
        for record in filtered {
            file.write_all(record.to_csv().as_bytes())?;
        }
        
        info!("Results saved with {} filtered records", filtered.len());
        Ok(())
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Records: {}, Processed: {}, Errors: {}",
            self.records.len(),
            self.processed_count,
            self.error_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 500.0, "2024-01-15T10:30:00Z");
        assert!(valid_record.validate().is_ok());

        let invalid_id = DataRecord::new(0, 500.0, "2024-01-15T10:30:00Z");
        assert!(invalid_id.validate().is_err());

        let invalid_value = DataRecord::new(1, 1500.0, "2024-01-15T10:30:00Z");
        assert!(invalid_value.validate().is_err());
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,100.5,2024-01-15T10:30:00Z").unwrap();
        writeln!(temp_file, "2,200.0,2024-01-15T11:00:00Z").unwrap();
        writeln!(temp_file, "3,50.0,2024-01-15T11:30:00Z").unwrap();
        
        processor.load_from_file(temp_file.path()).unwrap();
        
        let filtered = processor.filter_by_threshold(150.0);
        assert_eq!(filtered.len(), 1);
        
        let stats = processor.calculate_statistics();
        assert!((stats.0 - 116.833).abs() < 0.001);
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
    active: bool,
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
        let mut reader = Reader::from_reader(file);

        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold && record.active)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records.iter().find(|record| record.id == target_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,active").unwrap();
        writeln!(file, "1,alpha,42.5,true").unwrap();
        writeln!(file, "2,beta,18.3,false").unwrap();
        writeln!(file, "3,gamma,67.2,true").unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(test_file.path().to_str().unwrap()).unwrap();
        
        let filtered = processor.filter_by_value(40.0);
        assert_eq!(filtered.len(), 2);
        
        let avg = processor.calculate_average().unwrap();
        assert!((avg - 42.666).abs() < 0.001);
    }
}