
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
}

pub fn validate_timestamp(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.timestamp < 0 {
        return Err(ProcessingError::ValidationError(
            "Timestamp cannot be negative".to_string(),
        ));
    }
    Ok(())
}

pub fn normalize_values(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    if record.values.is_empty() {
        return Err(ProcessingError::TransformationError(
            "No values to normalize".to_string(),
        ));
    }

    let sum: f64 = record.values.iter().sum();
    if sum.abs() < f64::EPSILON {
        return Err(ProcessingError::TransformationError(
            "Cannot normalize zero vector".to_string(),
        ));
    }

    let normalized_values: Vec<f64> = record.values.iter().map(|&v| v / sum).collect();

    Ok(DataRecord {
        values: normalized_values,
        ..record
    })
}

pub fn add_processing_metadata(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let mut new_metadata = record.metadata.clone();
    new_metadata.insert(
        "processed_at".to_string(),
        chrono::Utc::now().timestamp().to_string(),
    );
    new_metadata.insert("record_id".to_string(), record.id.to_string());

    Ok(DataRecord {
        metadata: new_metadata,
        ..record
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_timestamp() {
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let invalid_record = DataRecord {
            id: 2,
            timestamp: -1,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(validate_timestamp(&valid_record).is_ok());
        assert!(validate_timestamp(&invalid_record).is_err());
    }

    #[test]
    fn test_normalize_values() {
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let normalized = normalize_values(record).unwrap();
        let sum: f64 = normalized.values.iter().sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_processor_pipeline() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(validate_timestamp);
        processor.add_transformation(normalize_values);
        processor.add_transformation(add_processing_metadata);

        let record = DataRecord {
            id: 42,
            timestamp: 1609459200,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert!(processed.metadata.contains_key("processed_at"));
        assert!(processed.metadata.contains_key("record_id"));
    }
}
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
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationFailed(
                "ID cannot be zero".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Values cannot be empty".to_string(),
            ));
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData(
                    "Values contain NaN or infinite numbers".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) -> Result<(), ProcessingError> {
        self.validate()?;

        let sum: f64 = self.values.iter().sum();
        if sum == 0.0 {
            return Err(ProcessingError::TransformationError(
                "Cannot normalize zero sum vector".to_string(),
            ));
        }

        for value in self.values.iter_mut() {
            *value /= sum;
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> Result<HashMap<String, f64>, ProcessingError> {
        self.validate()?;

        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;

        let variance: f64 = self
            .values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / count;

        let mut sorted_values = self.values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[count as usize / 2]
        };

        let mut stats = HashMap::new();
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("median".to_string(), median);

        Ok(stats)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<HashMap<String, f64>>, ProcessingError> {
    let mut results = Vec::new();

    for record in records {
        record.normalize()?;
        let stats = record.calculate_statistics()?;
        results.push(stats);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record_zero_id() {
        let record = DataRecord::new(0, vec![1.0, 2.0, 3.0]);
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(record.normalize().is_ok());
        
        let sum: f64 = record.values.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let stats = record.calculate_statistics().unwrap();
        
        assert_eq!(stats["count"], 5.0);
        assert_eq!(stats["sum"], 15.0);
        assert_eq!(stats["mean"], 3.0);
        assert_eq!(stats["median"], 3.0);
    }
}