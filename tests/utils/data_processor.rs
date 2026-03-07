
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
            let line_content = line?;
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
                
            if fields.iter().any(|f| f.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_num + 1).into());
            }
            
            records.push(fields);
        }
        
        Ok(records)
    }
    
    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to validate".into());
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
    }
    
    #[test]
    fn test_validation() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records).is_ok());
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
}

pub struct DataProcessor {
    validation_threshold: f64,
    normalization_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, normalization_factor: f64) -> Self {
        Self {
            validation_threshold,
            normalization_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationFailed(
                "Empty values array".to_string(),
            ));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationFailed(
                    "Invalid numeric value".to_string(),
                ));
            }

            if value.abs() > self.validation_threshold {
                return Err(ProcessingError::ValidationFailed(format!(
                    "Value {} exceeds threshold {}",
                    value, self.validation_threshold
                )));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if self.normalization_factor == 0.0 {
            return Err(ProcessingError::TransformationError(
                "Normalization factor cannot be zero".to_string(),
            ));
        }

        for value in &mut record.values {
            *value /= self.normalization_factor;
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        let mut all_values = Vec::new();

        for record in records {
            all_values.extend(&record.values);
        }

        if !all_values.is_empty() {
            let sum: f64 = all_values.iter().sum();
            let count = all_values.len() as f64;
            let mean = sum / count;

            let variance: f64 = all_values
                .iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>()
                / count;

            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("count".to_string(), count);
            stats.insert("sum".to_string(), sum);
        }

        stats
    }

    pub fn filter_records(
        &self,
        records: Vec<DataRecord>,
        predicate: impl Fn(&DataRecord) -> bool,
    ) -> Vec<DataRecord> {
        records.into_iter().filter(predicate).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 1.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 20.3, 30.7],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(10.0, 1.0);
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.5, 25.3, 30.7],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(1000.0, 10.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![100.0, 200.0, 300.0],
            metadata: HashMap::new(),
        };

        assert!(processor.normalize_values(&mut record).is_ok());
        assert_eq!(record.values, vec![10.0, 20.0, 30.0]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(header_result) = lines.next() {
            let header_line = header_result?;
            let headers: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();

            for line_result in lines {
                let line = line_result?;
                let values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
                
                if values.len() == headers.len() {
                    let mut record = HashMap::new();
                    for (i, header) in headers.iter().enumerate() {
                        record.insert(header.clone(), values[i].clone());
                    }
                    self.records.push(record);
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_average(&self, column_name: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value_str) = record.get(column_name) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }

    pub fn count_unique_values(&self, column_name: &str) -> HashMap<String, usize> {
        let mut frequency_map = HashMap::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column_name) {
                *frequency_map.entry(value.clone()).or_insert(0) += 1;
            }
        }
        
        frequency_map
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<HashMap<String, String>>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        self.records.iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_column_names(&self) -> Vec<String> {
        if let Some(first_record) = self.records.first() {
            first_record.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "name,age,salary").unwrap();
        writeln!(csv_data, "Alice,30,50000").unwrap();
        writeln!(csv_data, "Bob,25,45000").unwrap();
        writeln!(csv_data, "Charlie,35,60000").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_csv(csv_data.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let avg_age = processor.calculate_average("age");
        assert!(avg_age.is_some());
        assert!((avg_age.unwrap() - 30.0).abs() < 0.001);
        
        let unique_names = processor.count_unique_values("name");
        assert_eq!(unique_names.get("Alice"), Some(&1));
        
        let high_earners = processor.filter_records(|record| {
            record.get("salary").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) > 50000.0
        });
        assert_eq!(high_earners.len(), 1);
    }
}