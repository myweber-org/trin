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
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    config: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new(config: HashMap<String, String>) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError("Name cannot be empty".to_string()));
        }
        
        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError("Value must be non-negative".to_string()));
        }
        
        if record.tags.len() > 10 {
            return Err(ProcessingError::ValidationError("Too many tags".to_string()));
        }
        
        Ok(())
    }

    pub fn transform_record(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        let uppercase_name = record.name.to_uppercase();
        
        if uppercase_name.len() > 50 {
            return Err(ProcessingError::TransformationFailed(
                "Transformed name exceeds maximum length".to_string()
            ));
        }
        
        record.name = uppercase_name;
        
        if let Some(prefix) = self.config.get("value_prefix") {
            record.value = record.value + prefix.parse::<f64>().unwrap_or(0.0);
        }
        
        record.tags.retain(|tag| !tag.trim().is_empty());
        record.tags.sort();
        record.tags.dedup();
        
        Ok(())
    }

    pub fn process_batch(&self, records: &mut [DataRecord]) -> Vec<Result<DataRecord, ProcessingError>> {
        records.iter_mut()
            .map(|record| {
                self.validate_record(record)?;
                self.transform_record(record)?;
                Ok(record.clone())
            })
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }
        
        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let avg = sum / count;
        
        let max = records.iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);
        
        let min = records.iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);
        
        stats.insert("total_records".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("maximum".to_string(), max);
        stats.insert("minimum".to_string(), min);
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let config = HashMap::new();
        let processor = DataProcessor::new(config);
        
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(processor.validate_record(&valid_record).is_ok());
        
        let invalid_record = DataRecord {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            tags: vec![],
        };
        
        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_transformation() {
        let mut config = HashMap::new();
        config.insert("value_prefix".to_string(), "50".to_string());
        
        let processor = DataProcessor::new(config);
        
        let mut record = DataRecord {
            id: 1,
            name: "test record".to_string(),
            value: 100.0,
            tags: vec!["b".to_string(), "a".to_string(), "b".to_string()],
        };
        
        assert!(processor.transform_record(&mut record).is_ok());
        assert_eq!(record.name, "TEST RECORD");
        assert_eq!(record.value, 150.0);
        assert_eq!(record.tags, vec!["a", "b"]);
    }
}