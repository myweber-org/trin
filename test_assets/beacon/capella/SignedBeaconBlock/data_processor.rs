
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
    validation_rules: HashMap<String, Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule<F>(&mut self, name: &str, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.insert(name.to_string(), Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), Vec<ProcessingError>> {
        let mut errors = Vec::new();

        for (rule_name, rule_func) in &self.validation_rules {
            if let Err(err) = rule_func(record) {
                errors.push(err);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        if let Err(errors) = self.validate_record(&record) {
            return Err(ProcessingError::ValidationError(
                format!("Record validation failed with {} errors", errors.len())
            ));
        }

        for transform in &self.transformation_pipeline {
            record = transform(record)?;
        }

        Ok(record)
    }

    pub fn batch_process(&self, records: Vec<DataRecord>) -> (Vec<DataRecord>, Vec<ProcessingError>) {
        let mut successful = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.process_record(record) {
                Ok(processed) => successful.push(processed),
                Err(err) => errors.push(err),
            }
        }

        (successful, errors)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule("id_positive", |record| {
        if record.id == 0 {
            Err(ProcessingError::ValidationError(
                "ID must be greater than zero".to_string()
            ))
        } else {
            Ok(())
        }
    });

    processor.add_validation_rule("name_not_empty", |record| {
        if record.name.trim().is_empty() {
            Err(ProcessingError::ValidationError(
                "Name cannot be empty".to_string()
            ))
        } else {
            Ok(())
        }
    });

    processor.add_validation_rule("value_range", |record| {
        if record.value < 0.0 || record.value > 1000.0 {
            Err(ProcessingError::ValidationError(
                format!("Value {} out of range [0, 1000]", record.value)
            ))
        } else {
            Ok(())
        }
    });

    processor.add_transformation(|mut record| {
        record.name = record.name.trim().to_string();
        record.value = (record.value * 100.0).round() / 100.0;
        Ok(record)
    });

    processor.add_transformation(|mut record| {
        record.tags.sort();
        record.tags.dedup();
        Ok(record)
    });

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = create_default_processor();
        
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            tags: vec![],
        };

        assert!(processor.validate_record(&valid_record).is_ok());
        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_transformation() {
        let processor = create_default_processor();
        
        let record = DataRecord {
            id: 1,
            name: "  Test Data  ".to_string(),
            value: 123.456,
            tags: vec!["tag2".to_string(), "tag1".to_string(), "tag2".to_string()],
        };

        let processed = processor.process_record(record).unwrap();
        
        assert_eq!(processed.name, "Test Data");
        assert_eq!(processed.value, 123.46);
        assert_eq!(processed.tags, vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_batch_processing() {
        let processor = create_default_processor();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 100.0,
                tags: vec![],
            },
            DataRecord {
                id: 0,
                name: "".to_string(),
                value: -50.0,
                tags: vec![],
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 200.0,
                tags: vec![],
            },
        ];

        let (successful, errors) = processor.batch_process(records);
        
        assert_eq!(successful.len(), 2);
        assert_eq!(errors.len(), 1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64)> {
        let values: Vec<f64> = records
            .iter()
            .filter_map(|record| record.get(column_index).and_then(|s| s.parse::<f64>().ok()))
            .collect();

        if values.is_empty() {
            return None;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        
        Some((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,88.0").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_record(&["test".to_string(), "data".to_string()]));
        assert!(!processor.validate_record(&[]));
        assert!(!processor.validate_record(&["".to_string(), "value".to_string()]));
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.5".to_string()],
            vec!["20.0".to_string()],
            vec!["15.5".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&records, 0).unwrap();
        
        assert!((stats.0 - 15.333).abs() < 0.001);
        assert!((stats.1 - 4.041).abs() < 0.001);
    }
}