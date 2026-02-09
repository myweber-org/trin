
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
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

    pub fn batch_process(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records.into_iter().map(|record| self.process(record)).collect()
    }
}

fn validate_positive_values(record: &DataRecord) -> Result<(), ProcessingError> {
    for value in &record.values {
        if *value < 0.0 {
            return Err(ProcessingError::ValidationFailed(
                format!("Negative value found: {}", value)
            ));
        }
    }
    Ok(())
}

fn normalize_values(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let sum: f64 = record.values.iter().sum();
    if sum == 0.0 {
        return Err(ProcessingError::TransformationError(
            "Cannot normalize zero-sum vector".to_string()
        ));
    }

    let normalized_values: Vec<f64> = record.values.iter()
        .map(|v| v / sum)
        .collect();

    Ok(DataRecord {
        values: normalized_values,
        ..record
    })
}

fn add_processing_timestamp(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let mut new_metadata = record.metadata.clone();
    new_metadata.insert(
        "processed_at".to_string(),
        chrono::Utc::now().to_rfc3339(),
    );

    Ok(DataRecord {
        metadata: new_metadata,
        ..record
    })
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    processor.add_validation_rule(validate_positive_values);
    processor.add_transformation(normalize_values);
    processor.add_transformation(add_processing_timestamp);
    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_positive_values() {
        let record = DataRecord {
            id: 1,
            values: vec![1.0, 2.0, -1.0],
            metadata: HashMap::new(),
        };

        let result = validate_positive_values(&record);
        assert!(result.is_err());
    }

    #[test]
    fn test_normalization() {
        let record = DataRecord {
            id: 1,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let result = normalize_values(record).unwrap();
        let sum: f64 = result.values.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_full_processing() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 1,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert!(processed.metadata.contains_key("processed_at"));
    }
}