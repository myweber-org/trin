
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
        if record.name.is_empty() {
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

    pub fn transform_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        if let Some(prefix) = self.config.get("name_prefix") {
            transformed.name = format!("{}{}", prefix, transformed.name);
        }
        
        if let Some(factor_str) = self.config.get("value_multiplier") {
            if let Ok(factor) = factor_str.parse::<f64>() {
                transformed.value *= factor;
            } else {
                return Err(ProcessingError::TransformationFailed(
                    "Invalid multiplier in config".to_string()
                ));
            }
        }
        
        if let Some(default_tag) = self.config.get("default_tag") {
            if transformed.tags.is_empty() {
                transformed.tags.push(default_tag.clone());
            }
        }
        
        Ok(transformed)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed.push(transformed);
        }
        
        Ok(processed)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }
        
        let count = records.len() as f64;
        let sum: f64 = records.iter().map(|r| r.value).sum();
        let avg = sum / count;
        
        let variance: f64 = records.iter()
            .map(|r| (r.value - avg).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);
        
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
            name: "test".to_string(),
            value: 10.0,
            tags: vec!["tag1".to_string()],
        };
        
        assert!(processor.validate_record(&valid_record).is_ok());
        
        let invalid_record = DataRecord {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            tags: vec![],
        };
        
        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_transformation() {
        let mut config = HashMap::new();
        config.insert("name_prefix".to_string(), "pre_".to_string());
        config.insert("value_multiplier".to_string(), "2.0".to_string());
        
        let processor = DataProcessor::new(config);
        
        let record = DataRecord {
            id: 1,
            name: "data".to_string(),
            value: 5.0,
            tags: vec![],
        };
        
        let transformed = processor.transform_record(&record).unwrap();
        assert_eq!(transformed.name, "pre_data");
        assert_eq!(transformed.value, 10.0);
    }
}