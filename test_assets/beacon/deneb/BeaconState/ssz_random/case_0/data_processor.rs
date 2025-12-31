
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidCategory,
    RecordNotFound,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Value must be positive"),
            ProcessingError::InvalidCategory => write!(f, "Category cannot be empty"),
            ProcessingError::RecordNotFound => write!(f, "Record not found"),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        if record.value <= 0.0 {
            return Err(ProcessingError::InvalidValue);
        }
        if record.category.is_empty() {
            return Err(ProcessingError::InvalidCategory);
        }
        self.records.push(record);
        Ok(())
    }

    pub fn find_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
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
            value: 42.5,
            category: String::from("A"),
        };
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_value() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            value: -10.0,
            category: String::from("A"),
        };
        assert!(matches!(
            processor.add_record(record),
            Err(ProcessingError::InvalidValue)
        ));
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor
            .add_record(DataRecord {
                id: 1,
                value: 10.0,
                category: String::from("A"),
            })
            .unwrap();
        processor
            .add_record(DataRecord {
                id: 2,
                value: 20.0,
                category: String::from("B"),
            })
            .unwrap();
        assert_eq!(processor.calculate_average(), Some(15.0));
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor
            .add_record(DataRecord {
                id: 1,
                value: 10.0,
                category: String::from("A"),
            })
            .unwrap();
        processor
            .add_record(DataRecord {
                id: 2,
                value: 20.0,
                category: String::from("B"),
            })
            .unwrap();
        processor
            .add_record(DataRecord {
                id: 3,
                value: 30.0,
                category: String::from("A"),
            })
            .unwrap();

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "A"));
    }
}