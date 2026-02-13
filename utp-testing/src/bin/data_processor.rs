
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationFailed("ID cannot be zero".into()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::ValidationFailed("Timestamp cannot be negative".into()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::ValidationFailed("Values cannot be empty".into()));
        }
        
        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(DataError::ValidationFailed("Key cannot be empty".into()));
            }
            if !value.is_finite() {
                return Err(DataError::ValidationFailed(
                    format!("Value for key '{}' must be finite", key)
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> Result<(), DataError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(DataError::ValidationFailed(
                "Multiplier must be non-zero finite value".into()
            ));
        }
        
        for value in self.values.values_mut() {
            *value *= multiplier;
        }
        
        self.timestamp += 1;
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }
        
        let values: Vec<f64> = self.values.values().copied().collect();
        let count = values.len() as f64;
        
        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("count".into(), count);
        stats.insert("sum".into(), sum);
        stats.insert("mean".into(), mean);
        stats.insert("variance".into(), variance);
        stats.insert("min".into(), min);
        stats.insert("max".into(), max);
        
        stats
    }
}

pub fn process_records(records: Vec<DataRecord>, multiplier: f64) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for mut record in records {
        record.validate()?;
        record.transform(multiplier)?;
        processed.push(record);
    }
    
    Ok(processed)
}

pub fn filter_records_by_tag(records: Vec<DataRecord>, tag: &str) -> Vec<DataRecord> {
    records.into_iter()
        .filter(|record| record.tags.iter().any(|t| t == tag))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: HashMap::from([("temp".into(), 25.5)]),
            tags: vec!["sensor".into()],
        };
        
        assert!(record.validate().is_ok());
        
        record.id = 0;
        assert!(record.validate().is_err());
        
        record.id = 1;
        record.timestamp = -1;
        assert!(record.validate().is_err());
        
        record.timestamp = 1000;
        record.values.clear();
        assert!(record.validate().is_err());
    }
    
    #[test]
    fn test_record_transform() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: HashMap::from([("temp".into(), 25.5), ("pressure".into(), 1013.25)]),
            tags: vec![],
        };
        
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.values.get("temp"), Some(&51.0));
        assert_eq!(record.values.get("pressure"), Some(&2026.5));
        assert_eq!(record.timestamp, 1001);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: HashMap::from([
                ("a".into(), 1.0),
                ("b".into(), 2.0),
                ("c".into(), 3.0),
                ("d".into(), 4.0),
                ("e".into(), 5.0),
            ]),
            tags: vec![],
        };
        
        let stats = record.calculate_statistics();
        assert_eq!(stats.get("count"), Some(&5.0));
        assert_eq!(stats.get("sum"), Some(&15.0));
        assert_eq!(stats.get("mean"), Some(&3.0));
        assert_eq!(stats.get("min"), Some(&1.0));
        assert_eq!(stats.get("max"), Some(&5.0));
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

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        if data.is_empty() {
            return Err("No data available for statistics".into());
        }

        let mut values = Vec::new();
        for record in data {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }
            
            match record[column_index].parse::<f64>() {
                Ok(value) => values.push(value),
                Err(_) => return Err(format!("Invalid numeric value in column {}", column_index).into()),
            }
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,test1,10.5").unwrap();
        writeln!(temp_file, "2,test2,20.3").unwrap();
        writeln!(temp_file, "3,test3,15.7").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], vec!["1", "test1", "10.5"]);
    }

    #[test]
    fn test_statistics_calculation() {
        let data = vec![
            vec!["10.5".to_string(), "test1".to_string()],
            vec!["20.3".to_string(), "test2".to_string()],
            vec!["15.7".to_string(), "test3".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&data, 0).unwrap();
        
        let expected_mean = (10.5 + 20.3 + 15.7) / 3.0;
        assert!((stats.0 - expected_mean).abs() < 0.0001);
    }
}