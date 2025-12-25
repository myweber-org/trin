
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

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
    TransformationFailed(String),
    ValidationError(String),
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub require_timestamp: bool,
    pub allowed_metadata_keys: Vec<String>,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            max_values: 100,
            require_timestamp: true,
            allowed_metadata_keys: vec![],
        }
    }
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if self.config.require_timestamp && record.timestamp <= 0 {
            return Err(ProcessingError::ValidationError(
                "Timestamp must be positive".to_string(),
            ));
        }

        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationError(format!(
                "Too many values: {} exceeds maximum {}",
                record.values.len(),
                self.config.max_values
            )));
        }

        if !self.config.allowed_metadata_keys.is_empty() {
            for key in record.metadata.keys() {
                if !self.config.allowed_metadata_keys.contains(key) {
                    return Err(ProcessingError::ValidationError(format!(
                        "Metadata key '{}' is not allowed",
                        key
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn transform_record(
        &self,
        record: &DataRecord,
        transformation: &Transformation,
    ) -> Result<DataRecord, ProcessingError> {
        let mut transformed = record.clone();

        match transformation {
            Transformation::Normalize => {
                if let Some(max) = transformed.values.iter().copied().reduce(f64::max) {
                    if max != 0.0 {
                        for value in transformed.values.iter_mut() {
                            *value /= max;
                        }
                    }
                }
            }
            Transformation::Scale(factor) => {
                for value in transformed.values.iter_mut() {
                    *value *= factor;
                }
            }
            Transformation::FilterThreshold(threshold) => {
                transformed.values.retain(|&v| v >= *threshold);
            }
        }

        if transformed.values.is_empty() {
            return Err(ProcessingError::TransformationFailed(
                "All values filtered out".to_string(),
            ));
        }

        Ok(transformed)
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
        transformation: Option<Transformation>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());

        for record in records {
            self.validate_record(&record)?;

            let processed_record = if let Some(ref transform) = transformation {
                self.transform_record(&record, transform)?
            } else {
                record
            };

            processed.push(processed_record);
        }

        Ok(processed)
    }

    pub fn calculate_statistics(records: &[DataRecord]) -> ProcessingStatistics {
        let mut stats = ProcessingStatistics::default();

        for record in records {
            stats.total_records += 1;
            stats.total_values += record.values.len();

            if let Some(avg) = calculate_average(&record.values) {
                stats.average_value = (stats.average_value * (stats.total_records - 1) as f64 + avg)
                    / stats.total_records as f64;
            }

            for value in &record.values {
                if *value > stats.max_value {
                    stats.max_value = *value;
                }
                if *value < stats.min_value {
                    stats.min_value = *value;
                }
            }
        }

        stats
    }
}

#[derive(Debug, Clone)]
pub enum Transformation {
    Normalize,
    Scale(f64),
    FilterThreshold(f64),
}

#[derive(Debug, Clone)]
pub struct ProcessingStatistics {
    pub total_records: usize,
    pub total_values: usize,
    pub average_value: f64,
    pub min_value: f64,
    pub max_value: f64,
}

impl Default for ProcessingStatistics {
    fn default() -> Self {
        Self {
            total_records: 0,
            total_values: 0,
            average_value: 0.0,
            min_value: f64::MAX,
            max_value: f64::MIN,
        }
    }
}

fn calculate_average(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let sum: f64 = values.iter().sum();
    Some(sum / values.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let config = ProcessingConfig::default();
        let processor = DataProcessor::new(config);

        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let config = ProcessingConfig {
            max_values: 2,
            ..Default::default()
        };
        let processor = DataProcessor::new(config);

        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_scale() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let transformed = processor
            .transform_record(&record, &Transformation::Scale(2.0))
            .unwrap();

        assert_eq!(transformed.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2,
                values: vec![3.0, 4.0],
                metadata: HashMap::new(),
            },
        ];

        let stats = DataProcessor::calculate_statistics(&records);

        assert_eq!(stats.total_records, 2);
        assert_eq!(stats.total_values, 4);
        assert_eq!(stats.min_value, 1.0);
        assert_eq!(stats.max_value, 4.0);
    }
}