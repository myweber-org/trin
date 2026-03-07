use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: u64) -> Self {
        DataRecord {
            id,
            name,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value.is_finite() && self.timestamp > 0
    }
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
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let timestamp = match parts[3].parse::<u64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, name, value, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= threshold)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, 1234567890);
        assert!(valid_record.is_valid());

        let invalid_name = DataRecord::new(2, "".to_string(), 10.5, 1234567890);
        assert!(!invalid_name.is_valid());

        let invalid_value = DataRecord::new(3, "test".to_string(), f64::NAN, 1234567890);
        assert!(!invalid_value.is_valid());

        let invalid_timestamp = DataRecord::new(4, "test".to_string(), 10.5, 0);
        assert!(!invalid_timestamp.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,record1,10.5,1234567890").unwrap();
        writeln!(temp_file, "2,record2,20.3,1234567891").unwrap();
        writeln!(temp_file, "invalid,record3,not_a_number,1234567892").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.get_records().len(), 2);
    }

    #[test]
    fn test_filter_and_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "a".to_string(), 5.0, 1000));
        processor.records.push(DataRecord::new(2, "b".to_string(), 15.0, 1001));
        processor.records.push(DataRecord::new(3, "c".to_string(), 25.0, 1002));

        let filtered = processor.filter_by_value(10.0);
        assert_eq!(filtered.len(), 2);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 15.0).abs() < f64::EPSILON);
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
    let mut reader = Reader::from_reader(file);
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        validate_record(&record)?;
        records.push(record);
    }
    
    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    
    if !["A", "B", "C", "D"].contains(&record.category.as_str()) {
        return Err("Invalid category".to_string());
    }
    
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let average = sum / count;
    
    let max = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);
    
    let min = records.iter()
        .map(|r| r.value)
        .fold(f64::INFINITY, f64::min);
    
    (average, min, max)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
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

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();

        for (index, record) in records.iter().enumerate() {
            if record.is_empty() || record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }

        invalid_indices
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        let mut values = Vec::new();

        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
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
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["".to_string(), "c".to_string()],
            vec!["d".to_string(), "".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records);
        
        assert_eq!(invalid, vec![1, 2]);
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
        assert!((stats.2 - 4.509).abs() < 0.001);
    }
}