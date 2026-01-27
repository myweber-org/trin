
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidTimestamp(i64),
    EmptyCategory,
    SerializationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidTimestamp(t) => write!(f, "Invalid timestamp: {}", t),
            ProcessingError::EmptyCategory => write!(f, "Category cannot be empty"),
            ProcessingError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_enabled: bool,
    max_value: f64,
}

impl DataProcessor {
    pub fn new(validation_enabled: bool, max_value: f64) -> Self {
        DataProcessor {
            validation_enabled,
            max_value,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if self.validation_enabled {
            if record.value < 0.0 || record.value > self.max_value {
                return Err(ProcessingError::InvalidValue(record.value));
            }
            
            if record.timestamp < 0 {
                return Err(ProcessingError::InvalidTimestamp(record.timestamp));
            }
            
            if record.category.trim().is_empty() {
                return Err(ProcessingError::EmptyCategory);
            }
        }
        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> DataRecord {
        DataRecord {
            id: record.id,
            value: record.value * 1.1,
            timestamp: record.timestamp + 3600,
            category: record.category.to_uppercase(),
        }
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            let transformed = self.transform_record(&record);
            processed.push(transformed);
        }
        
        Ok(processed)
    }

    pub fn serialize_to_json(&self, records: &[DataRecord]) -> Result<String, ProcessingError> {
        serde_json::to_string(records)
            .map_err(|e| ProcessingError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_processing() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 1,
            value: 500.0,
            timestamp: 1625097600,
            category: "analytics".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
        
        let transformed = processor.transform_record(&record);
        assert_eq!(transformed.value, 550.0);
        assert_eq!(transformed.timestamp, 1625101200);
        assert_eq!(transformed.category, "ANALYTICS");
    }

    #[test]
    fn test_invalid_value() {
        let processor = DataProcessor::new(true, 1000.0);
        let record = DataRecord {
            id: 1,
            value: 1500.0,
            timestamp: 1625097600,
            category: "test".to_string(),
        };
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(ProcessingError::InvalidValue(1500.0))
        ));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(true, 1000.0);
        let records = vec![
            DataRecord {
                id: 1,
                value: 100.0,
                timestamp: 1625097600,
                category: "alpha".to_string(),
            },
            DataRecord {
                id: 2,
                value: 200.0,
                timestamp: 1625097600,
                category: "beta".to_string(),
            },
        ];
        
        let result = processor.process_batch(records);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].category, "ALPHA");
        assert_eq!(processed[1].category, "BETA");
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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut records = Vec::new();
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_num, line) in lines {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().any(|f| f.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_num + 1).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid data records found".into());
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("Empty record set".into());
        }

        let expected_len = records[0].len();
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", 
                    idx + 1, record.len(), expected_len).into());
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        let mut values = Vec::new();
        
        for record in records {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }
            
            match record[column_index].parse::<f64>() {
                Ok(value) => values.push(value),
                Err(_) => return Err(format!("Non-numeric value in column {}", column_index).into()),
            }
        }

        if values.is_empty() {
            return Err("No numeric values found".into());
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;

        Ok((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,87.2").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_validate_records() {
        let processor = DataProcessor::new(',', false);
        let valid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        assert!(processor.validate_records(&valid_records).is_ok());
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(',', false);
        let records = vec![
            vec!["10.0".to_string(), "20.0".to_string()],
            vec!["20.0".to_string(), "30.0".to_string()],
            vec!["30.0".to_string(), "40.0".to_string()],
        ];
        
        let result = processor.calculate_statistics(&records, 0);
        assert!(result.is_ok());
        let (mean, std_dev) = result.unwrap();
        assert!((mean - 20.0).abs() < 0.001);
        assert!((std_dev - 8.1649).abs() < 0.001);
    }
}