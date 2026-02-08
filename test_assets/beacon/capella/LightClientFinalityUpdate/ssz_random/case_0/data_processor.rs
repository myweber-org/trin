
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for rule in &self.validation_rules {
            rule(&record)?;
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());

        for record in records {
            match self.process(record) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        let mut processor = DataProcessor::new();

        processor.add_validation_rule(|record| {
            if record.values.is_empty() {
                Err(ProcessingError::ValidationFailed(
                    "Record must contain at least one value".to_string(),
                ))
            } else {
                Ok(())
            }
        });

        processor.add_validation_rule(|record| {
            if record.timestamp < 0 {
                Err(ProcessingError::ValidationFailed(
                    "Timestamp must be non-negative".to_string(),
                ))
            } else {
                Ok(())
            }
        });

        processor.add_transformation(|mut record| {
            let sum: f64 = record.values.iter().sum();
            let avg = sum / record.values.len() as f64;
            record
                .metadata
                .insert("average".to_string(), avg.to_string());
            Ok(record)
        });

        processor.add_transformation(|mut record| {
            record.values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            Ok(record)
        });

        processor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor_validation() {
        let processor = DataProcessor::default();
        let mut record = DataRecord {
            id: 1,
            timestamp: -1,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record.clone());
        assert!(result.is_err());

        record.timestamp = 1000;
        let result = processor.process(record);
        assert!(result.is_ok());
    }

    #[test]
    fn test_data_processor_transformation() {
        let processor = DataProcessor::default();
        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![3.0, 1.0, 2.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record).unwrap();
        assert_eq!(result.values, vec![1.0, 2.0, 3.0]);
        assert!(result.metadata.contains_key("average"));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::default();
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![3.0, 1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![5.0, 4.0],
                metadata: HashMap::new(),
            },
        ];

        let results = processor.batch_process(records).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].values, vec![1.0, 2.0, 3.0]);
        assert_eq!(results[1].values, vec![4.0, 5.0]);
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

#[derive(Default)]
pub struct ProcessingConfig {
    pub validate_on_load: bool,
    pub normalize_values: bool,
    pub max_tags_per_record: usize,
}

pub struct ProcessingStats {
    pub records_processed: u64,
    pub validation_errors: u64,
    pub transformation_errors: u64,
}

impl Default for ProcessingStats {
    fn default() -> Self {
        Self {
            records_processed: 0,
            validation_errors: 0,
            transformation_errors: 0,
        }
    }
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        Self {
            config,
            statistics: ProcessingStats::default(),
        }
    }

    pub fn process_record(&mut self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        self.statistics.records_processed += 1;

        if self.config.validate_on_load {
            self.validate_record(record)?;
        }

        if self.config.normalize_values {
            self.normalize_record(record)?;
        }

        if self.config.max_tags_per_record > 0 {
            self.limit_tags(record);
        }

        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string(),
            ));
        }

        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::ValidationError(
                "Record value must be a valid number".to_string(),
            ));
        }

        if record.id == 0 {
            return Err(ProcessingError::ValidationError(
                "Record ID must be non-zero".to_string(),
            ));
        }

        Ok(())
    }

    fn normalize_record(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        let original_value = record.value;
        
        record.value = record.value.clamp(0.0, 100.0);
        
        if original_value != record.value {
            record.tags.push("normalized".to_string());
        }

        Ok(())
    }

    fn limit_tags(&self, record: &mut DataRecord) {
        if record.tags.len() > self.config.max_tags_per_record {
            record.tags.truncate(self.config.max_tags_per_record);
            record.tags.push("truncated".to_string());
        }
    }

    pub fn batch_process(
        &mut self,
        records: &mut [DataRecord],
    ) -> Vec<Result<(), ProcessingError>> {
        records
            .iter_mut()
            .map(|record| self.process_record(record))
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let count = records.len() as f64;
        let sum: f64 = records.iter().map(|r| r.value).sum();
        let avg = sum / count;

        let variance: f64 = records
            .iter()
            .map(|r| (r.value - avg).powi(2))
            .sum::<f64>()
            / count;

        stats.insert("record_count".to_string(), count);
        stats.insert("value_sum".to_string(), sum);
        stats.insert("value_average".to_string(), avg);
        stats.insert("value_variance".to_string(), variance);
        stats.insert("value_min".to_string(), records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min));
        stats.insert("value_max".to_string(), records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max));

        stats
    }

    pub fn get_statistics(&self) -> &ProcessingStats {
        &self.statistics
    }

    pub fn reset_statistics(&mut self) {
        self.statistics = ProcessingStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let config = ProcessingConfig {
            validate_on_load: true,
            normalize_values: false,
            max_tags_per_record: 0,
        };

        let mut processor = DataProcessor::new(config);

        let mut valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            tags: vec![],
        };

        assert!(processor.process_record(&mut valid_record).is_ok());

        let mut invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: f64::NAN,
            tags: vec![],
        };

        assert!(processor.process_record(&mut invalid_record).is_err());
    }

    #[test]
    fn test_value_normalization() {
        let config = ProcessingConfig {
            validate_on_load: false,
            normalize_values: true,
            max_tags_per_record: 0,
        };

        let mut processor = DataProcessor::new(config);

        let mut high_value = DataRecord {
            id: 1,
            name: "High".to_string(),
            value: 150.0,
            tags: vec![],
        };

        processor.process_record(&mut high_value).unwrap();
        assert_eq!(high_value.value, 100.0);
        assert!(high_value.tags.contains(&"normalized".to_string()));

        let mut low_value = DataRecord {
            id: 2,
            name: "Low".to_string(),
            value: -50.0,
            tags: vec![],
        };

        processor.process_record(&mut low_value).unwrap();
        assert_eq!(low_value.value, 0.0);
        assert!(low_value.tags.contains(&"normalized".to_string()));
    }

    #[test]
    fn test_tag_limiting() {
        let config = ProcessingConfig {
            validate_on_load: false,
            normalize_values: false,
            max_tags_per_record: 3,
        };

        let mut processor = DataProcessor::new(config);

        let mut record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            tags: vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()],
        };

        processor.process_record(&mut record).unwrap();
        assert_eq!(record.tags.len(), 3);
        assert!(record.tags.contains(&"truncated".to_string()));
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_rules: Vec<Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: Vec::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.push(Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn validate(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        for rule in &self.validation_rules {
            rule(record)?;
        }
        Ok(())
    }

    pub fn process(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate(&record)?;

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records.into_iter().map(|record| self.process(record)).collect()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        let mut processor = DataProcessor::new();

        processor.add_validation_rule(|record| {
            if record.values.is_empty() {
                Err(ProcessingError::ValidationFailed("Empty values array".to_string()))
            } else {
                Ok(())
            }
        });

        processor.add_validation_rule(|record| {
            if record.timestamp < 0 {
                Err(ProcessingError::ValidationFailed("Negative timestamp".to_string()))
            } else {
                Ok(())
            }
        });

        processor.add_transformation(|mut record| {
            let sum: f64 = record.values.iter().sum();
            record.metadata.insert("values_sum".to_string(), sum.to_string());
            Ok(record)
        });

        processor.add_transformation(|mut record| {
            let avg = record.values.iter().sum::<f64>() / record.values.len() as f64;
            record.metadata.insert("values_avg".to_string(), avg.to_string());
            Ok(record)
        });

        processor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor_validation() {
        let processor = DataProcessor::default();
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate(&valid_record).is_ok());

        let invalid_record = DataRecord {
            id: 2,
            timestamp: -1,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate(&invalid_record).is_err());
    }

    #[test]
    fn test_data_processor_transformation() {
        let processor = DataProcessor::default();
        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let processed = processor.process(record).unwrap();
        assert_eq!(processed.metadata.get("values_sum").unwrap(), "6");
        assert_eq!(processed.metadata.get("values_avg").unwrap(), "2");
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::default();
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1625097600,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1625097600,
                values: vec![3.0, 4.0, 5.0],
                metadata: HashMap::new(),
            },
        ];

        let results = processor.batch_process(records);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
}