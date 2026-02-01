
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