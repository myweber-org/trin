
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ProcessingError {
    message: String,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Processing error: {}", self.message)
    }
}

impl Error for ProcessingError {}

impl ProcessingError {
    pub fn new(msg: &str) -> Self {
        ProcessingError {
            message: msg.to_string(),
        }
    }
}

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: i64) -> Result<Self, ProcessingError> {
        if value < 0.0 || value > 1000.0 {
            return Err(ProcessingError::new("Value must be between 0 and 1000"));
        }
        if timestamp < 0 {
            return Err(ProcessingError::new("Timestamp cannot be negative"));
        }
        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }
}

pub fn normalize_value(value: f64) -> f64 {
    (value - 500.0) / 500.0
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<f64>, ProcessingError> {
    if records.is_empty() {
        return Err(ProcessingError::new("No records to process"));
    }

    let mut results = Vec::with_capacity(records.len());
    for record in records {
        let normalized = normalize_value(record.value);
        results.push(normalized);
    }

    Ok(results)
}

pub fn calculate_statistics(values: &[f64]) -> (f64, f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;
    
    let variance: f64 = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, 1625097600);
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.timestamp, 1625097600);
    }

    #[test]
    fn test_invalid_value_record() {
        let record = DataRecord::new(1, -10.0, 1625097600);
        assert!(record.is_err());
    }

    #[test]
    fn test_normalize_value() {
        assert_eq!(normalize_value(500.0), 0.0);
        assert_eq!(normalize_value(1000.0), 1.0);
        assert_eq!(normalize_value(0.0), -1.0);
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord::new(1, 500.0, 1625097600).unwrap(),
            DataRecord::new(2, 750.0, 1625097601).unwrap(),
            DataRecord::new(3, 250.0, 1625097602).unwrap(),
        ];
        
        let result = process_records(records);
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], 0.0);
        assert_eq!(values[1], 0.5);
        assert_eq!(values[2], -0.5);
    }

    #[test]
    fn test_calculate_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = calculate_statistics(&values);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
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

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        Self {
            id,
            value,
            category: category.to_string(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
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
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, parts[2]);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
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

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
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

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A");
        assert!(valid_record.is_valid());

        let invalid_id = DataRecord::new(0, 10.5, "A");
        assert!(!invalid_id.is_valid());

        let invalid_value = DataRecord::new(1, -5.0, "A");
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(1, 10.5, "");
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,20.3,TypeB").unwrap();
        writeln!(temp_file, "3,15.7,TypeA").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.5, "TypeA"));
        processor.records.push(DataRecord::new(2, 20.3, "TypeB"));
        processor.records.push(DataRecord::new(3, 15.7, "TypeA"));

        let filtered = processor.filter_by_category("TypeA");
        assert_eq!(filtered.len(), 2);
        
        let filtered = processor.filter_by_category("TypeB");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A"));
        processor.records.push(DataRecord::new(2, 20.0, "B"));
        processor.records.push(DataRecord::new(3, 30.0, "C"));

        let stats = processor.get_statistics();
        assert_eq!(stats, (10.0, 30.0, 20.0));
        
        let avg = processor.calculate_average();
        assert_eq!(avg, Some(20.0));
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        assert_eq!(processor.calculate_average(), None);
        assert_eq!(processor.get_statistics(), (0.0, 0.0, 0.0));
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_data_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count.max(1.0);
    
    (mean, variance.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test1,10.5,true").unwrap();
        writeln!(temp_file, "2,Test2,-3.2,false").unwrap();
        writeln!(temp_file, "3,Test3,7.8,true").unwrap();

        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
        assert_eq!(records[1].value, 7.8);
    }

    #[test]
    fn test_statistics_calculation() {
        let test_records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];

        let (mean, std_dev) = calculate_statistics(&test_records);
        assert!((mean - 20.0).abs() < 0.001);
        assert!((std_dev - 8.164966).abs() < 0.001);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
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

            let category = parts[3].to_string();

            let record = DataRecord::new(id, name, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
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

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
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
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.3,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.7,CategoryA").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(
            1,
            "Test1".to_string(),
            10.0,
            "CategoryA".to_string(),
        ));
        processor.records.push(DataRecord::new(
            2,
            "Test2".to_string(),
            20.0,
            "CategoryB".to_string(),
        ));
        processor.records.push(DataRecord::new(
            3,
            "Test3".to_string(),
            30.0,
            "CategoryA".to_string(),
        ));

        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(
            1,
            "Test1".to_string(),
            10.0,
            "A".to_string(),
        ));
        processor.records.push(DataRecord::new(
            2,
            "Test2".to_string(),
            20.0,
            "B".to_string(),
        ));
        processor.records.push(DataRecord::new(
            3,
            "Test3".to_string(),
            30.0,
            "C".to_string(),
        ));

        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);
        assert_eq!(processor.calculate_average(), None);
        assert_eq!(processor.find_max_value(), None);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
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
            
            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[2].to_string();
            let valid = parts[3].parse::<bool>().unwrap_or(false);
            
            let record = DataRecord {
                id,
                value,
                category,
                valid,
            };
            
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.valid)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.filter_valid();
        if valid_records.is_empty() {
            return None;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
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
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,TypeA,true").unwrap();
        writeln!(temp_file, "2,20.3,TypeB,false").unwrap();
        writeln!(temp_file, "3,15.7,TypeA,true").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.count_records(), 3);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 13.1).abs() < 0.001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("TypeA").unwrap().len(), 2);
        assert_eq!(groups.get("TypeB").unwrap().len(), 1);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if let Some(value_str) = parts.get(0) {
                if let Ok(value) = value_str.parse::<f64>() {
                    self.data.push(value);
                }
            }
            
            if let Some(category) = parts.get(1) {
                *self.frequency_map.entry(category.to_string()).or_insert(0) += 1;
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, u32> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let count = self.data.len();
        let unique_categories = self.frequency_map.len();
        
        format!("Data points: {}, Mean: {:.2}, Categories: {}", count, mean, unique_categories)
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
        writeln!(temp_file, "10.5,category_a").unwrap();
        writeln!(temp_file, "20.3,category_b").unwrap();
        writeln!(temp_file, "15.7,category_a").unwrap();
        writeln!(temp_file, "invalid,category_c").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.data.len(), 3);
        assert_eq!(processor.calculate_mean(), Some(15.5));
        
        let frequencies = processor.get_frequency_distribution();
        assert_eq!(frequencies.get("category_a"), Some(&2));
        assert_eq!(frequencies.get("category_b"), Some(&1));
    }
}