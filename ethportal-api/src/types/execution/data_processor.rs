
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidCategory(String),
    EmptyDataset,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidCategory(c) => write!(f, "Invalid category: {}", c),
            ProcessingError::EmptyDataset => write!(f, "Dataset is empty"),
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
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.category.is_empty() || record.category.len() > 50 {
            return Err(ProcessingError::InvalidCategory(record.category));
        }

        self.records.push(record);
        Ok(())
    }

    pub fn calculate_average(&self) -> Result<f64, ProcessingError> {
        if self.records.is_empty() {
            return Err(ProcessingError::EmptyDataset);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Ok(sum / self.records.len() as f64)
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

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = values.iter().sum::<f64>() / values.len() as f64;

        (min, max, avg)
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
            value: 100.0,
            category: "test".to_string(),
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
            category: "test".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor
            .add_record(DataRecord {
                id: 1,
                value: 50.0,
                category: "A".to_string(),
            })
            .unwrap();
        processor
            .add_record(DataRecord {
                id: 2,
                value: 100.0,
                category: "B".to_string(),
            })
            .unwrap();

        assert_eq!(processor.calculate_average().unwrap(), 75.0);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor
            .add_record(DataRecord {
                id: 1,
                value: 10.0,
                category: "cat1".to_string(),
            })
            .unwrap();
        processor
            .add_record(DataRecord {
                id: 2,
                value: 20.0,
                category: "cat2".to_string(),
            })
            .unwrap();
        processor
            .add_record(DataRecord {
                id: 3,
                value: 30.0,
                category: "cat1".to_string(),
            })
            .unwrap();

        let filtered = processor.filter_by_category("cat1");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "cat1"));
    }
}