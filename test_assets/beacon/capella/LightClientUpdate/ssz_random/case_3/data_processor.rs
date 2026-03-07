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
        
        if let Some(max_tags) = self.config.get("max_tags") {
            if let Ok(max) = max_tags.parse::<usize>() {
                if record.tags.len() > max {
                    return Err(ProcessingError::ValidationError(
                        format!("Exceeded maximum tags limit: {}", max)
                    ));
                }
            }
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
                    "Invalid multiplier configuration".to_string()
                ));
            }
        }
        
        if let Some(default_tag) = self.config.get("default_tag") {
            if !transformed.tags.contains(default_tag) {
                transformed.tags.push(default_tag.clone());
            }
        }
        
        Ok(transformed)
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record)?;
            processed.push(transformed);
        }
        
        Ok(processed)
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
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
        config.insert("name_prefix".to_string(), "PREFIX_".to_string());
        config.insert("value_multiplier".to_string(), "2.0".to_string());
        config.insert("default_tag".to_string(), "processed".to_string());
        
        let processor = DataProcessor::new(config);
        
        let record = DataRecord {
            id: 1,
            name: "data".to_string(),
            value: 5.0,
            tags: vec!["original".to_string()],
        };
        
        let transformed = processor.transform_record(&record).unwrap();
        
        assert_eq!(transformed.name, "PREFIX_data");
        assert_eq!(transformed.value, 10.0);
        assert!(transformed.tags.contains(&"processed".to_string()));
    }
    
    #[test]
    fn test_statistics() {
        let records = vec![
            DataRecord { id: 1, name: "A".to_string(), value: 10.0, tags: vec![] },
            DataRecord { id: 2, name: "B".to_string(), value: 20.0, tags: vec![] },
            DataRecord { id: 3, name: "C".to_string(), value: 30.0, tags: vec![] },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
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

fn validate_timestamp(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.timestamp < 0 {
        return Err(ProcessingError::ValidationError(
            "Timestamp cannot be negative".to_string(),
        ));
    }
    Ok(())
}

fn validate_values(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.values.is_empty() {
        return Err(ProcessingError::ValidationError(
            "Values vector cannot be empty".to_string(),
        ));
    }

    for value in &record.values {
        if value.is_nan() || value.is_infinite() {
            return Err(ProcessingError::ValidationError(
                "Values contain NaN or infinite numbers".to_string(),
            ));
        }
    }
    Ok(())
}

fn normalize_values(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let sum: f64 = record.values.iter().sum();
    if sum == 0.0 {
        return Err(ProcessingError::TransformationError(
            "Cannot normalize zero-sum vector".to_string(),
        ));
    }

    let normalized_values: Vec<f64> = record.values.iter().map(|v| v / sum).collect();
    Ok(DataRecord {
        values: normalized_values,
        ..record
    })
}

fn add_processing_flag(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let mut metadata = record.metadata;
    metadata.insert("processed".to_string(), "true".to_string());
    metadata.insert("processor_version".to_string(), "1.0.0".to_string());

    Ok(DataRecord { metadata, ..record })
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    processor.add_validation_rule(validate_timestamp);
    processor.add_validation_rule(validate_values);
    processor.add_transformation(normalize_values);
    processor.add_transformation(add_processing_flag);
    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_validation_success() {
        let record = create_test_record();
        let processor = create_default_processor();
        assert!(processor.process(record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let record = DataRecord {
            id: 2,
            timestamp: -1,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };
        let processor = create_default_processor();
        assert!(processor.process(record).is_err());
    }

    #[test]
    fn test_normalization() {
        let record = create_test_record();
        let processor = create_default_processor();
        let result = processor.process(record).unwrap();
        let sum: f64 = result.values.iter().sum();
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_metadata_update() {
        let record = create_test_record();
        let processor = create_default_processor();
        let result = processor.process(record).unwrap();
        assert_eq!(result.metadata.get("processed").unwrap(), "true");
        assert_eq!(result.metadata.get("processor_version").unwrap(), "1.0.0");
    }
}