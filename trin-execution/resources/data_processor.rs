
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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
    records: Vec<DataRecord>,
    category_stats: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        self.update_statistics(&record);
        self.records.push(record);
        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string(),
            ));
        }

        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError(
                "Record value cannot be negative".to_string(),
            ));
        }

        if record.category.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record category cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn update_statistics(&mut self, record: &DataRecord) {
        let category_total = self.category_stats
            .entry(record.category.clone())
            .or_insert(0.0);
        *category_total += record.value;
    }

    pub fn transform_values<F>(&mut self, transformer: F) -> Result<(), ProcessingError>
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            let original_value = record.value;
            record.value = transformer(record.value);
            
            if record.value.is_nan() || record.value.is_infinite() {
                return Err(ProcessingError::TransformationError(
                    format!("Transformation produced invalid value for record {}", record.id)
                ));
            }
            
            let category_total = self.category_stats
                .get_mut(&record.category)
                .ok_or_else(|| ProcessingError::TransformationError(
                    format!("Category {} not found in statistics", record.category)
                ))?;
            
            *category_total += record.value - original_value;
        }
        Ok(())
    }

    pub fn get_category_average(&self, category: &str) -> Option<f64> {
        let total = self.category_stats.get(category)?;
        let count = self.records.iter()
            .filter(|r| r.category == category)
            .count() as f64;
        
        if count > 0.0 {
            Some(total / count)
        } else {
            None
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_err());
        assert_eq!(processor.record_count(), 0);
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 20.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 30.0, category: "B".to_string() },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        processor.transform_values(|x| x * 2.0).unwrap();
        
        assert_eq!(processor.total_value(), 120.0);
        assert_eq!(processor.get_category_average("A"), Some(30.0));
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated_data = self.validate_data(data)?;
        let processed = self.transform_data(&validated_data);
        
        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let mut validated = Vec::with_capacity(data.len());
        
        for &value in data {
            if !value.is_finite() {
                return Err(format!("Invalid numeric value: {}", value));
            }
            validated.push(value);
        }
        
        Ok(validated)
    }

    fn transform_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = self.calculate_mean(data);
        data.iter()
            .map(|&x| (x - mean).abs())
            .collect()
    }

    fn calculate_mean(&self, data: &[f64]) -> f64 {
        let sum: f64 = data.iter().sum();
        sum / data.len() as f64
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_valid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), data.len());
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_numeric_data("invalid", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let first_result = processor.process_numeric_data("cached", &data).unwrap();
        let second_result = processor.process_numeric_data("cached", &data).unwrap();
        
        assert_eq!(first_result, second_result);
        assert_eq!(processor.get_cache_size(), 1);
    }
}