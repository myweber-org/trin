
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: u64) -> Self {
        Self {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
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

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let timestamp = match parts[3].parse::<u64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, category, timestamp);
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

    pub fn get_statistics(&self) -> Statistics {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len();

        if count == 0 {
            return Statistics::empty();
        }

        let sum: f64 = values.iter().sum();
        let avg = sum / count as f64;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let variance: f64 = values.iter().map(|&v| (v - avg).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Statistics {
            count,
            average: avg,
            minimum: min,
            maximum: max,
            standard_deviation: std_dev,
        }
    }

    pub fn sort_by_value(&mut self) {
        self.records.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub average: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub standard_deviation: f64,
}

impl Statistics {
    pub fn empty() -> Self {
        Self {
            count: 0,
            average: 0.0,
            minimum: 0.0,
            maximum: 0.0,
            standard_deviation: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test".to_string(), 1234567890);
        assert!(valid_record.is_valid());

        let invalid_id = DataRecord::new(0, 42.5, "test".to_string(), 1234567890);
        assert!(!invalid_id.is_valid());

        let invalid_value = DataRecord::new(1, f64::NAN, "test".to_string(), 1234567890);
        assert!(!invalid_value.is_valid());

        let empty_category = DataRecord::new(1, 42.5, "".to_string(), 1234567890);
        assert!(!empty_category.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert!(processor.is_empty());

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,category_a,1000").unwrap();
        writeln!(temp_file, "2,20.5,category_b,2000").unwrap();
        writeln!(temp_file, "3,30.5,category_a,3000").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.len(), 3);

        let category_a = processor.filter_by_category("category_a");
        assert_eq!(category_a.len(), 2);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 20.5).abs() < f64::EPSILON);

        let stats = processor.get_statistics();
        assert_eq!(stats.count, 3);
        assert!((stats.minimum - 10.5).abs() < f64::EPSILON);
        assert!((stats.maximum - 30.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sorting() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 30.0, "test".to_string(), 1000));
        processor.records.push(DataRecord::new(2, 10.0, "test".to_string(), 2000));
        processor.records.push(DataRecord::new(3, 20.0, "test".to_string(), 3000));

        processor.sort_by_value();
        assert!((processor.records[0].value - 10.0).abs() < f64::EPSILON);
        assert!((processor.records[1].value - 20.0).abs() < f64::EPSILON);
        assert!((processor.records[2].value - 30.0).abs() < f64::EPSILON);
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
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: u64) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
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
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let timestamp = match parts[3].parse::<u64>() {
                Ok(ts) => ts,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, category, timestamp);
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

    pub fn get_stats(&self) -> (f64, f64, f64) {
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
        let valid_record = DataRecord::new(1, 42.5, "test".to_string(), 1234567890);
        assert!(valid_record.is_valid());

        let invalid_id = DataRecord::new(0, 42.5, "test".to_string(), 1234567890);
        assert!(!invalid_id.is_valid());

        let invalid_value = DataRecord::new(1, f64::NAN, "test".to_string(), 1234567890);
        assert!(!invalid_value.is_valid());

        let empty_category = DataRecord::new(1, 42.5, "".to_string(), 1234567890);
        assert!(!empty_category.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.record_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,category_a,1000").unwrap();
        writeln!(temp_file, "2,20.5,category_b,2000").unwrap();
        writeln!(temp_file, "3,30.5,category_a,3000").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.record_count(), 3);

        let category_a_records = processor.filter_by_category("category_a");
        assert_eq!(category_a_records.len(), 2);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 20.5).abs() < f64::EPSILON);

        let (min, max, avg_stats) = processor.get_stats();
        assert!((min - 10.5).abs() < f64::EPSILON);
        assert!((max - 30.5).abs() < f64::EPSILON);
        assert!((avg_stats - 20.5).abs() < f64::EPSILON);

        processor.clear();
        assert_eq!(processor.record_count(), 0);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: u32,
    name: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "id,name,value,category\n1,ItemA,10.5,Alpha\n2,ItemB,-3.2,Beta\n3,,15.0,Alpha"
        )
        .unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_csv(file.path()).unwrap();

        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.validate_records().len(), 2);
        assert_eq!(processor.calculate_average(), Some(12.75));
        
        let groups = processor.group_by_category();
        assert_eq!(groups.get("Alpha").unwrap().len(), 2);
        assert_eq!(groups.get("Beta").unwrap().len(), 1);
    }
}
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        
        if self.values.is_empty() {
            return Err("Values cannot be empty".to_string());
        }
        
        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }
        
        Ok(())
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }
    
    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let all_values: Vec<f64> = records.iter()
        .flat_map(|r| r.values.iter())
        .copied()
        .collect();
    
    if !all_values.is_empty() {
        let sum: f64 = all_values.iter().sum();
        let count = all_values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = all_values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
        
        if let Some(max) = all_values.iter().copied().reduce(f64::max) {
            stats.insert("max".to_string(), max);
        }
        
        if let Some(min) = all_values.iter().copied().reduce(f64::min) {
            stats.insert("min".to_string(), min);
        }
    }
    
    stats
}

pub fn filter_records<F>(records: Vec<DataRecord>, predicate: F) -> Vec<DataRecord>
where
    F: Fn(&DataRecord) -> bool,
{
    records.into_iter().filter(predicate).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.5)
            .add_value(37.2)
            .add_metadata("source", "sensor_a");
        
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert_eq!(record.values.len(), 2);
        assert_eq!(record.metadata.get("source"), Some(&"sensor_a".to_string()));
    }

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1000,
            values: vec![1.0],
            metadata: HashMap::new(),
        };
        
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![4.0, 5.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = calculate_statistics(&records);
        
        assert_eq!(stats.get("mean"), Some(&3.0));
        assert_eq!(stats.get("total_records"), Some(&2.0));
        assert_eq!(stats.get("total_values"), Some(&5.0));
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            DataRecord::new(1, 1000),
            DataRecord::new(2, 2000),
            DataRecord::new(3, 3000),
        ];
        
        let filtered = filter_records(records, |r| r.id > 1);
        
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.id > 1));
    }
}