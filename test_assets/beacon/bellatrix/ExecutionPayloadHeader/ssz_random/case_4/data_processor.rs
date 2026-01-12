
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
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

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Result<Self, ProcessingError> {
        if values.is_empty() {
            return Err(ProcessingError::InvalidData("Values cannot be empty".to_string()));
        }
        
        if values.iter().any(|&v| v.is_nan() || v.is_infinite()) {
            return Err(ProcessingError::InvalidData("Values contain NaN or infinite numbers".to_string()));
        }
        
        Ok(Self {
            id,
            values,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.values.len() > 1000 {
            return Err(ProcessingError::ValidationError("Too many values".to_string()));
        }
        
        Ok(())
    }
    
    pub fn normalize(&mut self) -> Result<(), ProcessingError> {
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return Err(ProcessingError::TransformationError("Cannot normalize constant values".to_string()));
        }
        
        for value in &mut self.values {
            *value = (*value - min) / (max - min);
        }
        
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), *self.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        stats.insert("max".to_string(), *self.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        stats.insert("sum".to_string(), sum);
        
        stats
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
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }
    
    pub fn process_all(&mut self) -> Result<(), ProcessingError> {
        for record in &mut self.records {
            record.normalize()?;
        }
        Ok(())
    }
    
    pub fn get_aggregated_stats(&self) -> HashMap<String, f64> {
        let mut aggregated = HashMap::new();
        let mut total_mean = 0.0;
        let mut total_variance = 0.0;
        
        for record in &self.records {
            let stats = record.calculate_statistics();
            total_mean += stats.get("mean").unwrap_or(&0.0);
            total_variance += stats.get("variance").unwrap_or(&0.0);
        }
        
        let count = self.records.len() as f64;
        aggregated.insert("average_mean".to_string(), total_mean / count);
        aggregated.insert("average_variance".to_string(), total_variance / count);
        aggregated.insert("total_records".to_string(), count);
        
        aggregated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.values.len(), 3);
    }
    
    #[test]
    fn test_record_validation() {
        let record = DataRecord::new(0, vec![1.0]).unwrap();
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        record.normalize().unwrap();
        assert_eq!(record.values[0], 0.0);
        assert_eq!(record.values[2], 1.0);
    }
    
    #[test]
    fn test_statistics() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]).unwrap();
        let stats = record.calculate_statistics();
        assert_eq!(stats.get("mean").unwrap(), &2.0);
        assert_eq!(stats.get("sum").unwrap(), &6.0);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
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

pub struct ProcessingConfig {
    pub max_values: usize,
    pub require_timestamp: bool,
    pub allowed_metadata_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationError(
                format!("Record exceeds maximum allowed values: {}", self.config.max_values)
            ));
        }

        if self.config.require_timestamp && record.timestamp <= 0 {
            return Err(ProcessingError::ValidationError(
                "Record must have a valid positive timestamp".to_string()
            ));
        }

        for key in record.metadata.keys() {
            if !self.config.allowed_metadata_keys.contains(key) {
                return Err(ProcessingError::ValidationError(
                    format!("Metadata key '{}' is not allowed", key)
                ));
            }
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        transformed.values = record.values
            .iter()
            .map(|&value| {
                if value.is_nan() || value.is_infinite() {
                    Err(ProcessingError::TransformationError(
                        "Cannot transform NaN or infinite values".to_string()
                    ))
                } else {
                    Ok(value * 2.0)
                }
            })
            .collect::<Result<Vec<f64>, ProcessingError>>()?;

        transformed.metadata.insert(
            "processed".to_string(),
            "true".to_string()
        );

        transformed.metadata.insert(
            "transformation_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string()
        );

        Ok(transformed)
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed_records.push(transformed);
        }
        
        Ok(processed_records)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);

        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.clone())
            .collect();

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let count = all_values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = all_values
                .iter()
                .map(|&value| (value - mean).powi(2))
                .sum::<f64>() / count;

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("min".to_string(), all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
            stats.insert("max".to_string(), all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_values: 10,
            require_timestamp: true,
            allowed_metadata_keys: vec!["source".to_string(), "version".to_string()],
        }
    }

    fn create_valid_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: {
                let mut map = HashMap::new();
                map.insert("source".to_string(), "test".to_string());
                map
            },
        }
    }

    #[test]
    fn test_validate_valid_record() {
        let processor = DataProcessor::new(create_test_config());
        let record = create_valid_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_invalid_timestamp() {
        let processor = DataProcessor::new(create_test_config());
        let mut record = create_valid_record();
        record.timestamp = 0;
        
        match processor.validate_record(&record) {
            Err(ProcessingError::ValidationError(_)) => (),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(create_test_config());
        let record = create_valid_record();
        let transformed = processor.transform_record(&record).unwrap();
        
        assert_eq!(transformed.values, vec![2.0, 4.0, 6.0]);
        assert_eq!(transformed.metadata.get("processed"), Some(&"true".to_string()));
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![4.0, 5.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats.get("total_records"), Some(&2.0));
        assert_eq!(stats.get("total_values"), Some(&5.0));
        assert_eq!(stats.get("mean"), Some(&3.0));
        assert_eq!(stats.get("min"), Some(&1.0));
        assert_eq!(stats.get("max"), Some(&5.0));
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    data: Vec<Vec<String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            self.data.push(row);
        }
        
        Ok(())
    }

    pub fn validate_data(&self) -> bool {
        if self.data.is_empty() {
            return false;
        }
        
        let header_len = self.data[0].len();
        for row in &self.data[1..] {
            if row.len() != header_len {
                return false;
            }
        }
        
        true
    }

    pub fn get_column(&self, index: usize) -> Option<Vec<String>> {
        if self.data.is_empty() || index >= self.data[0].len() {
            return None;
        }
        
        let mut column = Vec::new();
        for row in &self.data {
            if let Some(value) = row.get(index) {
                column.push(value.clone());
            }
        }
        
        Some(column)
    }

    pub fn row_count(&self) -> usize {
        self.data.len()
    }

    pub fn column_count(&self) -> usize {
        if self.data.is_empty() {
            0
        } else {
            self.data[0].len()
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
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.row_count(), 3);
        assert_eq!(processor.column_count(), 3);
        assert!(processor.validate_data());
        
        let names = processor.get_column(0).unwrap();
        assert_eq!(names, vec!["name", "Alice", "Bob"]);
    }
}