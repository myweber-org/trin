
use csv::Reader;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value < 0.0 {
            return Err(format!("Invalid value {} for record {}", record.value, record.id).into());
        }
        
        records.push(record);
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_csv() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,value,category").unwrap();
        writeln!(file, "1,Test1,10.5,A").unwrap();
        writeln!(file, "2,Test2,20.3,B").unwrap();
        
        let result = process_csv_data(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "X".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Y".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub require_timestamp: bool,
    pub allowed_metadata_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationError(format!(
                "Too many values: {} > {}",
                record.values.len(),
                self.config.max_values
            )));
        }

        if self.config.require_timestamp && record.timestamp <= 0 {
            return Err(ProcessingError::ValidationError(
                "Invalid timestamp".to_string(),
            ));
        }

        for key in record.metadata.keys() {
            if !self.config.allowed_metadata_keys.contains(key) {
                return Err(ProcessingError::ValidationError(format!(
                    "Disallowed metadata key: {}",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn transform_record(
        &self,
        record: DataRecord,
    ) -> Result<TransformedRecord, ProcessingError> {
        self.validate_record(&record)?;

        let sum: f64 = record.values.iter().sum();
        let avg = if !record.values.is_empty() {
            sum / record.values.len() as f64
        } else {
            0.0
        };

        let normalized_values: Vec<f64> = if !record.values.is_empty() && sum != 0.0 {
            record.values.iter().map(|&v| v / sum).collect()
        } else {
            record.values
        };

        Ok(TransformedRecord {
            original_id: record.id,
            timestamp: record.timestamp,
            value_count: record.values.len(),
            value_sum: sum,
            value_average: avg,
            normalized_values,
            metadata_count: record.metadata.len(),
        })
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<TransformedRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        for record in records {
            match self.transform_record(record) {
                Ok(transformed) => results.push(transformed),
                Err(e) => return Err(e),
            }
        }
        Ok(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedRecord {
    pub original_id: u64,
    pub timestamp: i64,
    pub value_count: usize,
    pub value_sum: f64,
    pub value_average: f64,
    pub normalized_values: Vec<f64>,
    pub metadata_count: usize,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        ProcessingConfig {
            max_values: 100,
            require_timestamp: true,
            allowed_metadata_keys: vec![
                "source".to_string(),
                "version".to_string(),
                "type".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: {
                let mut map = HashMap::new();
                map.insert("source".to_string(), "test".to_string());
                map.insert("version".to_string(), "1.0".to_string());
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
        let mut config = ProcessingConfig::default();
        config.max_values = 3;
        let processor = DataProcessor::new(config);

        let mut record = create_test_record();
        record.values = vec![1.0; 10];

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let record = create_test_record();
        let result = processor.transform_record(record).unwrap();

        assert_eq!(result.original_id, 1);
        assert_eq!(result.value_count, 5);
        assert_eq!(result.value_sum, 15.0);
        assert_eq!(result.value_average, 3.0);
        assert_eq!(result.normalized_values.len(), 5);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let records = vec![create_test_record(), create_test_record()];
        let results = processor.batch_process(records).unwrap();

        assert_eq!(results.len(), 2);
        for result in results {
            assert_eq!(result.value_sum, 15.0);
        }
    }
}