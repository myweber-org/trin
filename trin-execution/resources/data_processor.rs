
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