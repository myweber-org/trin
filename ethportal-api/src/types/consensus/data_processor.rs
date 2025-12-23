
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

        for &value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationFailed(
                    "Values contain NaN or infinite numbers".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) -> Result<(), ProcessingError> {
        self.validate()?;

        let min = self
            .values
            .iter()
            .fold(f64::INFINITY, |acc, &x| acc.min(x));
        let max = self
            .values
            .iter()
            .fold(f64::NEG_INFINITY, |acc, &x| acc.max(x));

        if (max - min).abs() < f64::EPSILON {
            return Err(ProcessingError::TransformationError(
                "Cannot normalize constant data".to_string(),
            ));
        }

        for value in &mut self.values {
            *value = (*value - min) / (max - min);
        }

        self.add_metadata(
            "normalization_applied".to_string(),
            "true".to_string(),
        );

        Ok(())
    }

    pub fn calculate_statistics(&self) -> Result<HashMap<String, f64>, ProcessingError> {
        self.validate()?;

        let mut stats = HashMap::new();
        let count = self.values.len() as f64;

        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;

        let variance: f64 = self
            .values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);

        if let Some(&min) = self.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("min".to_string(), min);
        }

        if let Some(&max) = self.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("max".to_string(), max);
        }

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
        assert_eq!(record.values, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        let stats = record.calculate_statistics().unwrap();
        
        assert!((stats["mean"] - 2.0).abs() < f64::EPSILON);
        assert!((stats["variance"] - 0.6666666666666666).abs() < f64::EPSILON);
        assert_eq!(stats["count"], 3.0);
        assert_eq!(stats["sum"], 6.0);
    }
}