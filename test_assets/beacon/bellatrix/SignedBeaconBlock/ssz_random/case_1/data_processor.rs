
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.validate_record(&record)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), String> {
        if record.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if record.value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if record.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
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

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,type_a").unwrap();
        writeln!(temp_file, "2,20.3,type_b").unwrap();
        writeln!(temp_file, "3,15.7,type_a").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.001);
        
        let filtered = processor.filter_by_category("type_a");
        assert_eq!(filtered.len(), 2);
    }
}
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
        Self {
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

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule(|record| {
        if record.values.is_empty() {
            Err(ProcessingError::ValidationFailed(
                "Record must contain values".to_string(),
            ))
        } else {
            Ok(())
        }
    });

    processor.add_validation_rule(|record| {
        if record.timestamp < 0 {
            Err(ProcessingError::ValidationFailed(
                "Timestamp cannot be negative".to_string(),
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
        record.values = record
            .values
            .iter()
            .map(|&v| v.max(0.0).min(100.0))
            .collect();
        Ok(record)
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let processor = create_default_processor();

        let record = DataRecord {
            id: 1,
            timestamp: 1625097600,
            values: vec![10.0, 20.0, 30.0, 40.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record).unwrap();
        assert_eq!(result.values, vec![10.0, 20.0, 30.0, 40.0]);
        assert_eq!(
            result.metadata.get("average").unwrap(),
            &"25".to_string()
        );
    }

    #[test]
    fn test_validation_failure() {
        let processor = create_default_processor();

        let record = DataRecord {
            id: 2,
            timestamp: -1,
            values: vec![50.0, 60.0],
            metadata: HashMap::new(),
        };

        let result = processor.process(record);
        assert!(matches!(
            result,
            Err(ProcessingError::ValidationFailed(_))
        ));
    }

    #[test]
    fn test_batch_processing() {
        let processor = create_default_processor();

        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1625097600,
                values: vec![10.0, 90.0, 110.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 1625097601,
                values: vec![-10.0, 50.0, 150.0],
                metadata: HashMap::new(),
            },
        ];

        let results = processor.batch_process(records).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].values, vec![10.0, 90.0, 100.0]);
        assert_eq!(results[1].values, vec![0.0, 50.0, 100.0]);
    }
}