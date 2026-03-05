
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
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

            let active = match parts[3].to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                _ => false,
            };

            let record = Record::new(id, name, value, active);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&Record> {
        self.records.iter().find(|record| record.id == target_id)
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
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -50.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,ItemA,100.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,75.25,false").unwrap();
        writeln!(temp_file, "3,ItemC,50.0,true").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.calculate_total(), 225.75);
        assert_eq!(processor.filter_active().len(), 2);
        
        let found = processor.find_by_id(2);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "ItemB");
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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
            let valid = parts[3].parse::<bool>().unwrap_or(false);

            self.records.push(DataRecord {
                id,
                value,
                category,
                valid,
            });

            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn average_value(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,alpha,true").unwrap();
        writeln!(temp_file, "2,20.3,beta,false").unwrap();
        writeln!(temp_file, "3,15.7,alpha,true").unwrap();
        
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 2);
        
        let avg = processor.average_value().unwrap();
        assert!((avg - 13.1).abs() < 0.001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.get("alpha").unwrap().len(), 2);
        assert_eq!(groups.get("beta").unwrap().len(), 1);
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
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = Reader::from_reader(file);

        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_statistics(&self) -> (usize, f64, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();
        
        let min = self.records
            .iter()
            .map(|r| r.value)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
            
        let max = self.records
            .iter()
            .map(|r| r.value)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        (count, avg, min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,15.2,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,8.7,Category1").unwrap();
        writeln!(temp_file, "4,ItemD,12.3,Category2").unwrap();
        temp_file
    }

    #[test]
    fn test_load_and_filter() {
        let temp_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        let category1_items = processor.filter_by_category("Category1");
        assert_eq!(category1_items.len(), 2);
        
        let category2_items = processor.filter_by_category("Category2");
        assert_eq!(category2_items.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let temp_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        let avg = processor.calculate_average();
        assert!((avg - 11.675).abs() < 0.001);
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.calculate_average(), 0.0);
        assert_eq!(processor.find_max_value(), None);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub valid_records: Vec<DataRecord>,
    pub invalid_records: Vec<DataRecord>,
    pub statistics: ProcessingStats,
}

#[derive(Debug)]
pub struct ProcessingStats {
    pub total_processed: usize,
    pub validation_errors: usize,
    pub transformation_errors: usize,
    pub processing_time_ms: u128,
}

pub struct DataProcessor {
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        DataProcessor {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<ProcessingResult, Box<dyn Error>> {
        let start_time = std::time::Instant::now();
        let mut valid_records = Vec::new();
        let mut invalid_records = Vec::new();
        let mut validation_errors = 0;
        let mut transformation_errors = 0;

        for mut record in records {
            if self.validate_record(&record) {
                match self.transform_record(&mut record) {
                    Ok(_) => valid_records.push(record),
                    Err(_) => {
                        transformation_errors += 1;
                        invalid_records.push(record);
                    }
                }
            } else {
                validation_errors += 1;
                invalid_records.push(record);
            }
        }

        let processing_time = start_time.elapsed().as_millis();

        Ok(ProcessingResult {
            valid_records,
            invalid_records,
            statistics: ProcessingStats {
                total_processed: valid_records.len() + invalid_records.len(),
                validation_errors,
                transformation_errors,
                processing_time_ms: processing_time,
            },
        })
    }

    fn validate_record(&self, record: &DataRecord) -> bool {
        if record.id == 0 {
            return false;
        }

        if record.timestamp <= 0 {
            return false;
        }

        if record.values.is_empty() {
            return false;
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return false;
            }

            if value.abs() > self.validation_threshold {
                return false;
            }
        }

        true
    }

    fn transform_record(&self, record: &mut DataRecord) -> Result<(), Box<dyn Error>> {
        for value in record.values.iter_mut() {
            *value *= self.transformation_factor;
            
            if value.is_nan() || value.is_infinite() {
                return Err("Transformation produced invalid value".into());
            }
        }

        record.metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string(),
        );

        record.metadata.insert(
            "transformation_factor".to_string(),
            self.transformation_factor.to_string(),
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_validation() {
        let processor = DataProcessor::new(1000.0, 2.0);
        
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        let invalid_record = DataRecord {
            id: 0,
            timestamp: -1,
            values: vec![f64::NAN],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_processor_transformation() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let result = processor.transform_record(&mut record);
        assert!(result.is_ok());
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
        assert!(record.metadata.contains_key("processed_timestamp"));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1234567890,
                values: vec![10.0, 20.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 0,
                timestamp: 1234567890,
                values: vec![30.0, 40.0],
                metadata: HashMap::new(),
            },
        ];

        let result = processor.process_batch(records).unwrap();
        assert_eq!(result.valid_records.len(), 1);
        assert_eq!(result.invalid_records.len(), 1);
        assert_eq!(result.statistics.validation_errors, 1);
    }
}