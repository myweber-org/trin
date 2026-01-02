
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("Empty values array")]
    EmptyValues,
    #[error("NaN value detected at index {0}")]
    NaNValue(usize),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

pub struct DataProcessor {
    config: ProcessingConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub min_timestamp: i64,
    pub max_timestamp: i64,
    pub require_metadata: bool,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        Self { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.timestamp < self.config.min_timestamp 
            || record.timestamp > self.config.max_timestamp {
            return Err(DataError::InvalidTimestamp(record.timestamp));
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for (i, &value) in record.values.iter().enumerate() {
            if value.is_nan() {
                return Err(DataError::NaNValue(i));
            }
        }

        if self.config.require_metadata && record.metadata.is_empty() {
            return Err(DataError::ValidationFailed(
                "Metadata is required but empty".to_string()
            ));
        }

        Ok(())
    }

    pub fn normalize_values(&self, values: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return vec![0.0; values.len()];
        }

        values.iter()
            .map(|&v| (v - min) / (max - min))
            .collect()
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
        let mut processed = Vec::with_capacity(records.len());
        
        for record in records {
            self.validate_record(&record)?;
            
            let normalized_values = self.normalize_values(&record.values);
            
            let processed_record = DataRecord {
                values: normalized_values,
                ..record
            };
            
            processed.push(processed_record);
        }
        
        Ok(processed)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter()
            .map(|r| r.values.len())
            .sum();
        
        let sum_all: f64 = records.iter()
            .flat_map(|r| r.values.iter())
            .sum();
        
        let mean = sum_all / total_values as f64;
        
        let variance: f64 = records.iter()
            .flat_map(|r| r.values.iter())
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / total_values as f64;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
        
        stats
    }
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            max_values: 1000,
            min_timestamp: 0,
            max_timestamp: i64::MAX,
            require_metadata: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_record() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let record = DataRecord {
            id: 1,
            timestamp: 1000,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_invalid_timestamp() {
        let processor = DataProcessor::new(ProcessingConfig {
            min_timestamp: 1000,
            max_timestamp: 2000,
            ..ProcessingConfig::default()
        });
        
        let record = DataRecord {
            id: 1,
            timestamp: 500,
            values: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };
        
        assert!(matches!(
            processor.validate_record(&record),
            Err(DataError::InvalidTimestamp(500))
        ));
    }

    #[test]
    fn test_normalize_values() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = processor.normalize_values(&values);
        
        assert_eq!(normalized[0], 0.0);
        assert_eq!(normalized[4], 1.0);
        assert!(normalized[2] > 0.4 && normalized[2] < 0.6);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(ProcessingConfig::default());
        let records = vec![
            DataRecord {
                id: 1,
                timestamp: 1000,
                values: vec![1.0, 2.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                timestamp: 2000,
                values: vec![3.0, 4.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = processor.calculate_statistics(&records);
        
        assert_eq!(stats["mean"], 2.5);
        assert_eq!(stats["total_records"], 2.0);
        assert_eq!(stats["total_values"], 4.0);
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
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
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

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.valid)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
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
            if record.valid {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn count_valid(&self) -> usize {
        self.filter_valid().len()
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
        writeln!(temp_file, "1,10.5,category_a,true").unwrap();
        writeln!(temp_file, "2,20.3,category_b,true").unwrap();
        writeln!(temp_file, "3,invalid,category_a,false").unwrap();
        writeln!(temp_file, "4,15.7,category_a,true").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        assert_eq!(processor.count_records(), 3);
        assert_eq!(processor.count_valid(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 15.4).abs() < 0.001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("category_a").unwrap().len(), 1);
        assert_eq!(groups.get("category_b").unwrap().len(), 1);
    }
}