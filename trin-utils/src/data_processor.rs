
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
    config: ProcessingConfig,
}

pub struct ProcessingConfig {
    pub max_value: f64,
    pub min_value: f64,
    pub allowed_tags: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value > self.config.max_value {
            return Err(ProcessingError::ValidationError(
                format!("Value {} exceeds maximum {}", record.value, self.config.max_value)
            ));
        }

        if record.value < self.config.min_value {
            return Err(ProcessingError::ValidationError(
                format!("Value {} below minimum {}", record.value, self.config.min_value)
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

    pub fn transform_record(&self, record: DataRecord) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();
        
        transformed.value = (transformed.value * 100.0).round() / 100.0;
        
        transformed.tags = transformed.tags
            .into_iter()
            .map(|tag| tag.to_lowercase())
            .collect();

        if transformed.name.is_empty() {
            return Err(ProcessingError::TransformationFailed(
                "Record name cannot be empty".to_string()
            ));
        }

        Ok(transformed)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::new();
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(record)?;
            processed.push(transformed);
        }

        Ok(processed)
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

        stats.insert("average".to_string(), avg);
        stats.insert("maximum".to_string(), max);
        stats.insert("minimum".to_string(), min);
        stats.insert("total".to_string(), sum);
        stats.insert("count".to_string(), count);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_value: 1000.0,
            min_value: 0.0,
            allowed_tags: vec!["important".to_string(), "normal".to_string(), "test".to_string()],
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 500.0,
            tags: vec!["important".to_string()],
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_fails_on_high_value() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 1500.0,
            tags: vec!["important".to_string()],
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(create_test_config());
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 123.456,
            tags: vec!["IMPORTANT".to_string(), "TEST".to_string()],
        };

        let transformed = processor.transform_record(record).unwrap();
        assert_eq!(transformed.value, 123.46);
        assert_eq!(transformed.tags, vec!["important".to_string(), "test".to_string()]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 100.0,
                tags: vec!["normal".to_string()],
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 200.0,
                tags: vec!["important".to_string()],
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 300.0,
                tags: vec!["test".to_string()],
            },
        ];

        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats.get("average"), Some(&200.0));
        assert_eq!(stats.get("maximum"), Some(&300.0));
        assert_eq!(stats.get("minimum"), Some(&100.0));
        assert_eq!(stats.get("total"), Some(&600.0));
        assert_eq!(stats.get("count"), Some(&3.0));
    }
}