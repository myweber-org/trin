
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

    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}
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

pub struct DataProcessor {
    threshold: f64,
    multiplier: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64, multiplier: f64) -> Result<Self, ProcessingError> {
        if threshold <= 0.0 {
            return Err(ProcessingError::new("Threshold must be positive"));
        }
        if multiplier <= 0.0 {
            return Err(ProcessingError::new("Multiplier must be positive"));
        }
        
        Ok(DataProcessor {
            threshold,
            multiplier,
        })
    }
    
    pub fn process_value(&self, value: f64) -> Result<f64, ProcessingError> {
        if value < 0.0 {
            return Err(ProcessingError::new("Value cannot be negative"));
        }
        
        if value > self.threshold {
            Ok(value * self.multiplier)
        } else {
            Ok(value)
        }
    }
    
    pub fn process_batch(&self, values: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        let mut results = Vec::with_capacity(values.len());
        
        for &value in values {
            let processed = self.process_value(value)?;
            results.push(processed);
        }
        
        Ok(results)
    }
    
    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64, f64), ProcessingError> {
        if values.is_empty() {
            return Err(ProcessingError::new("Cannot calculate statistics for empty dataset"));
        }
        
        let processed = self.process_batch(values)?;
        
        let sum: f64 = processed.iter().sum();
        let mean = sum / processed.len() as f64;
        
        let variance: f64 = processed.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / processed.len() as f64;
        
        let max = processed.iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Ok((mean, variance.sqrt(), max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(10.0, 2.0);
        assert!(processor.is_ok());
    }
    
    #[test]
    fn test_invalid_threshold() {
        let processor = DataProcessor::new(0.0, 2.0);
        assert!(processor.is_err());
    }
    
    #[test]
    fn test_process_value_below_threshold() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let result = processor.process_value(5.0);
        assert_eq!(result.unwrap(), 5.0);
    }
    
    #[test]
    fn test_process_value_above_threshold() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let result = processor.process_value(15.0);
        assert_eq!(result.unwrap(), 30.0);
    }
    
    #[test]
    fn test_process_batch() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let values = vec![5.0, 15.0, 8.0, 20.0];
        let result = processor.process_batch(&values).unwrap();
        assert_eq!(result, vec![5.0, 30.0, 8.0, 40.0]);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let values = vec![5.0, 15.0, 8.0];
        let (mean, std_dev, max) = processor.calculate_statistics(&values).unwrap();
        
        let expected_mean = (5.0 + 30.0 + 8.0) / 3.0;
        assert!((mean - expected_mean).abs() < 1e-10);
        assert!(max == 30.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: String,
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

            let timestamp = parts[2].to_string();

            self.records.push(DataRecord {
                id,
                value,
                timestamp,
            });

            count += 1;
        }

        Ok(count)
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= threshold)
            .cloned()
            .collect()
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,2023-01-01T10:00:00").unwrap();
        writeln!(temp_file, "2,20.3,2023-01-01T11:00:00").unwrap();
        writeln!(temp_file, "3,15.7,2023-01-01T12:00:00").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.get_record_count(), 3);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.1);

        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);

        processor.clear();
        assert_eq!(processor.get_record_count(), 0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead};
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

    pub fn data_summary(&self) -> String {
        format!(
            "Data points: {}, Source: {}",
            self.data.len(),
            self.metadata.get("source").unwrap_or(&"Unknown".to_string())
        )
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
        writeln!(temp_file, "id,value").unwrap();
        writeln!(temp_file, "1,10.5").unwrap();
        writeln!(temp_file, "2,20.3").unwrap();
        writeln!(temp_file, "3,15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats["count"], 3.0);
        assert!((stats["mean"] - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
    }
}