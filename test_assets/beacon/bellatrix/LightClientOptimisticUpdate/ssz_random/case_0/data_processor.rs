use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord {
            id,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn display(&self) -> String {
        format!("ID: {}, Value: {:.2}, Category: {}", self.id, self.value, self.category)
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            eprintln!("Warning: Line {} has invalid format", line_num + 1);
            continue;
        }

        let id = match parts[0].parse::<u32>() {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Warning: Invalid ID on line {}", line_num + 1);
                continue;
            }
        };

        let value = match parts[1].parse::<f64>() {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Warning: Invalid value on line {}", line_num + 1);
                continue;
            }
        };

        let category = parts[2].trim().to_string();
        let record = DataRecord::new(id, value, category);

        if record.is_valid() {
            records.push(record);
        } else {
            eprintln!("Warning: Invalid record on line {}", line_num + 1);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_valid());
        assert_eq!(record.display(), "ID: 1, Value: 42.50, Category: test");
    }

    #[test]
    fn test_invalid_record() {
        let record1 = DataRecord::new(0, 42.5, "test".to_string());
        let record2 = DataRecord::new(1, -1.0, "test".to_string());
        let record3 = DataRecord::new(1, 42.5, "".to_string());
        
        assert!(!record1.is_valid());
        assert!(!record2.is_valid());
        assert!(!record3.is_valid());
    }

    #[test]
    fn test_process_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,42.5,category_a")?;
        writeln!(temp_file, "2,37.8,category_b")?;
        writeln!(temp_file, "# This is a comment")?;
        writeln!(temp_file, "")?;
        writeln!(temp_file, "3,29.1,category_c")?;

        let records = process_csv_file(temp_file.path().to_str().unwrap())?;
        assert_eq!(records.len(), 3);
        
        let stats = calculate_statistics(&records);
        assert!((stats.0 - 36.46666666666667).abs() < 0.0001);
        
        Ok(())
    }

    #[test]
    fn test_statistics_empty() {
        let records: Vec<DataRecord> = Vec::new();
        let stats = calculate_statistics(&records);
        assert_eq!(stats, (0.0, 0.0, 0.0));
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!(stats.0 > 0.0);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();

            self.records.push(DataRecord {
                id,
                value,
                category,
            });

            count += 1;
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.record_count(), 3);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 10.5);
        assert_eq!(stats.1, 20.3);
    }
}use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue,
    InvalidCategory,
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid numeric value"),
            ProcessingError::InvalidCategory => write!(f, "Invalid category"),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
    allowed_categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(threshold: f64, allowed_categories: Vec<String>) -> Self {
        DataProcessor {
            threshold,
            allowed_categories,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::InvalidValue);
        }

        if !self.allowed_categories.contains(&record.category) {
            return Err(ProcessingError::InvalidCategory);
        }

        if record.value < 0.0 {
            return Err(ProcessingError::ValidationFailed(
                "Negative values not allowed".to_string(),
            ));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &DataRecord) -> Option<DataRecord> {
        if record.value > self.threshold {
            Some(DataRecord {
                id: record.id,
                value: record.value * 0.9,
                category: record.category.clone(),
            })
        } else {
            None
        }
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ProcessingError>> {
        records
            .into_iter()
            .map(|record| {
                self.validate_record(&record)?;
                if let Some(transformed) = self.transform_record(&record) {
                    Ok(transformed)
                } else {
                    Ok(record)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(
            100.0,
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
        );
        let record = DataRecord {
            id: 1,
            value: 50.0,
            category: "A".to_string(),
        };
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_category() {
        let processor = DataProcessor::new(
            100.0,
            vec!["A".to_string(), "B".to_string()],
        );
        let record = DataRecord {
            id: 1,
            value: 50.0,
            category: "X".to_string(),
        };
        assert!(matches!(
            processor.validate_record(&record),
            Err(ProcessingError::InvalidCategory)
        ));
    }

    #[test]
    fn test_transform_above_threshold() {
        let processor = DataProcessor::new(100.0, vec!["TEST".to_string()]);
        let record = DataRecord {
            id: 1,
            value: 150.0,
            category: "TEST".to_string(),
        };
        let transformed = processor.transform_record(&record).unwrap();
        assert_eq!(transformed.value, 135.0);
    }

    #[test]
    fn test_process_batch() {
        let processor = DataProcessor::new(
            50.0,
            vec!["CAT1".to_string(), "CAT2".to_string()],
        );
        
        let records = vec![
            DataRecord {
                id: 1,
                value: 30.0,
                category: "CAT1".to_string(),
            },
            DataRecord {
                id: 2,
                value: 60.0,
                category: "CAT2".to_string(),
            },
            DataRecord {
                id: 3,
                value: -10.0,
                category: "CAT1".to_string(),
            },
        ];

        let results = processor.process_batch(records);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(matches!(results[2], Err(ProcessingError::ValidationFailed(_))));
    }
}