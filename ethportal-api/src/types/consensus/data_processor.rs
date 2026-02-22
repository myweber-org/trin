
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset_name: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field {} contains invalid values", rule.field_name));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&value| {
                let mut transformed = value;
                
                for rule in &self.validation_rules {
                    if value < rule.min_value {
                        transformed = rule.min_value;
                    } else if value > rule.max_value {
                        transformed = rule.max_value;
                    }
                }
                
                transformed * 1.05
            })
            .collect();

        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStatistics> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&value| {
                    let diff = mean - value;
                    diff * diff
                })
                .sum::<f64>() / count;
            
            DatasetStatistics {
                mean,
                variance,
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0),
                count: data.len(),
            }
        })
    }
}

pub struct DatasetStatistics {
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let rule = ValidationRule::new("temperature", -50.0, 150.0, true);
        processor.add_validation_rule(rule);
        
        let test_data = vec![25.0, 30.0, 35.0, 40.0];
        let result = processor.process_dataset("test_set", test_data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_data("test_set").unwrap().len(), 4);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let test_data = vec![10.0, 20.0, 30.0, 40.0];
        
        processor.process_dataset("stats_test", test_data).unwrap();
        let stats = processor.calculate_statistics("stats_test").unwrap();
        
        assert_eq!(stats.mean, 26.25);
        assert_eq!(stats.count, 4);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
            }
        }
        
        self.metadata.insert("source".to_string(), filepath.to_string());
        self.metadata.insert("loaded_at".to_string(), chrono::Local::now().to_rfc3339());
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }
        
        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);
        
        stats
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }

    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
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
        writeln!(temp_file, "id,value").unwrap();
        writeln!(temp_file, "1,10.5").unwrap();
        writeln!(temp_file, "2,20.3").unwrap();
        writeln!(temp_file, "3,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats["mean"] - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    MetadataTooLarge,
}

pub struct DataProcessor {
    max_metadata_size: usize,
    min_timestamp: i64,
}

impl DataProcessor {
    pub fn new(max_metadata_size: usize, min_timestamp: i64) -> Self {
        DataProcessor {
            max_metadata_size,
            min_timestamp,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }

        if record.timestamp < self.min_timestamp {
            return Err(ValidationError::InvalidTimestamp);
        }

        if record.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }

        let total_metadata_size: usize = record
            .metadata
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum();

        if total_metadata_size > self.max_metadata_size {
            return Err(ValidationError::MetadataTooLarge);
        }

        Ok(())
    }

    pub fn transform_values(&self, record: &mut DataRecord, multiplier: f64) {
        for value in record.values.iter_mut() {
            *value *= multiplier;
        }
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let sum_all: f64 = records
            .iter()
            .flat_map(|r| r.values.iter())
            .sum();

        let avg = sum_all / total_values as f64;

        let variance: f64 = records
            .iter()
            .flat_map(|r| r.values.iter())
            .map(|v| (v - avg).powi(2))
            .sum::<f64>()
            / total_values as f64;

        stats.insert("record_count".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);

        stats
    }

    pub fn filter_by_timestamp(&self, records: Vec<DataRecord>, start: i64, end: i64) -> Vec<DataRecord> {
        records
            .into_iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::from([
                ("source".to_string(), "test".to_string()),
                ("version".to_string(), "1.0".to_string()),
            ]),
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(100, 0);
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_id() {
        let processor = DataProcessor::new(100, 0);
        let mut record = create_test_record();
        record.id = 0;
        assert_eq!(processor.validate_record(&record), Err(ValidationError::InvalidId));
    }

    #[test]
    fn test_transform_values() {
        let processor = DataProcessor::new(100, 0);
        let mut record = create_test_record();
        processor.transform_values(&mut record, 2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(100, 0);
        let records = vec![create_test_record(), create_test_record()];
        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats.get("record_count"), Some(&2.0));
        assert_eq!(stats.get("total_values"), Some(&6.0));
        assert_eq!(stats.get("average"), Some(&2.0));
    }

    #[test]
    fn test_filter_by_timestamp() {
        let processor = DataProcessor::new(100, 0);
        let mut record1 = create_test_record();
        let mut record2 = create_test_record();
        record1.timestamp = 500;
        record2.timestamp = 1500;
        
        let records = vec![record1, record2];
        let filtered = processor.filter_by_timestamp(records, 1000, 2000);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].timestamp, 1500);
    }
}