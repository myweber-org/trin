
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Transformation failed: {0}")]
    TransformationFailed(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: HashMap<String, f64>,
    pub tags: Vec<String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError(
                "ID cannot be zero".to_string(),
            ));
        }

        if self.timestamp < 0 {
            return Err(ProcessingError::ValidationError(
                "Timestamp cannot be negative".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Values cannot be empty".to_string(),
            ));
        }

        for (key, value) in &self.values {
            if key.trim().is_empty() {
                return Err(ProcessingError::ValidationError(
                    "Key cannot be empty".to_string(),
                ));
            }
            if !value.is_finite() {
                return Err(ProcessingError::ValidationError(
                    format!("Value for key '{}' must be finite", key),
                ));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&mut self) -> Result<(), ProcessingError> {
        let sum: f64 = self.values.values().sum();
        if sum.abs() < f64::EPSILON {
            return Err(ProcessingError::TransformationFailed(
                "Cannot normalize zero sum".to_string(),
            ));
        }

        for value in self.values.values_mut() {
            *value /= sum;
        }

        Ok(())
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> HashMap<String, f64> {
        self.values
            .iter()
            .filter(|(_, &value)| value >= threshold)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}

pub struct DataProcessor {
    pub records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn process_all(&mut self) -> Result<(), ProcessingError> {
        for record in &mut self.records {
            record.normalize_values()?;
        }
        Ok(())
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }

        let total_records = self.records.len() as f64;
        
        let total_values: usize = self.records.iter().map(|r| r.values.len()).sum();
        stats.insert("average_values_per_record".to_string(), total_values as f64 / total_records);

        let mut total_sum = 0.0;
        let mut count = 0;
        
        for record in &self.records {
            for value in record.values.values() {
                total_sum += value;
                count += 1;
            }
        }

        if count > 0 {
            stats.insert("average_value".to_string(), total_sum / count as f64);
        }

        stats
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.tags.contains(&tag.to_string()))
            .collect()
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([("temp".to_string(), 25.5)]),
            tags: vec!["sensor".to_string()],
        };

        assert!(record.validate().is_ok());

        record.id = 0;
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_normalize_values() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("a".to_string(), 1.0),
                ("b".to_string(), 2.0),
                ("c".to_string(), 3.0),
            ]),
            tags: vec![],
        };

        assert!(record.normalize_values().is_ok());
        
        let sum: f64 = record.values.values().sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_filter_by_threshold() {
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: HashMap::from([
                ("a".to_string(), 0.1),
                ("b".to_string(), 0.5),
                ("c".to_string(), 0.9),
            ]),
            tags: vec![],
        };

        let filtered = record.filter_by_threshold(0.5);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains_key("b"));
        assert!(filtered.contains_key("c"));
    }
}use std::error::Error;
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

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
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

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
    }

    pub fn calculate_column_averages(&self, records: &[Vec<String>]) -> Result<Vec<f64>, Box<dyn Error>> {
        if records.is_empty() {
            return Ok(Vec::new());
        }

        let column_count = records[0].len();
        let mut sums = vec![0.0; column_count];
        let mut counts = vec![0; column_count];

        for record in records {
            for (i, field) in record.iter().enumerate() {
                if let Ok(value) = field.parse::<f64>() {
                    sums[i] += value;
                    counts[i] += 1;
                }
            }
        }

        let averages: Vec<f64> = sums
            .iter()
            .zip(counts.iter())
            .map(|(&sum, &count)| if count > 0 { sum / count as f64 } else { 0.0 })
            .collect();

        Ok(averages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.5"]);
        
        let invalid = processor.validate_records(&records);
        assert_eq!(invalid, vec![2]);
        
        let averages = processor.calculate_column_averages(&records[0..2]).unwrap();
        assert_eq!(averages[1], 27.5);
        assert_eq!(averages[2], 47500.25);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_data_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,A").unwrap();
        writeln!(temp_file, "2,Test2,15.3,B").unwrap();
        
        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "X".to_string() },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 30.0);
        assert_eq!(mean, 15.0);
        assert!(std_dev - 5.0 < 0.0001);
    }

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Y".to_string() },
        ];
        
        let filtered = filter_by_category(records, "X");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
}