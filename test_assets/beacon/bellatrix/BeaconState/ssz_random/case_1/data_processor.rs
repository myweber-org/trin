
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
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

            let category = parts[2].to_string();

            let record = DataRecord::new(id, value, category);
            self.add_record(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.total_value += record.get_value();
        }
        self.records.push(record);
    }

    pub fn get_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn get_total_value(&self) -> f64 {
        self.total_value
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.get_valid_records();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.get_value()).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.is_valid() && r.category == category)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "type_a".to_string());
        assert!(valid_record.is_valid());

        let invalid_value = DataRecord::new(2, -10.0, "type_b".to_string());
        assert!(!invalid_value.is_valid());

        let invalid_category = DataRecord::new(3, 15.0, "".to_string());
        assert!(!invalid_category.is_valid());
    }

    #[test]
    fn test_processor_operations() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, 10.0, "alpha".to_string()));
        processor.add_record(DataRecord::new(2, 20.0, "beta".to_string()));
        processor.add_record(DataRecord::new(3, 30.0, "alpha".to_string()));
        
        assert_eq!(processor.get_valid_records().len(), 3);
        assert_eq!(processor.get_total_value(), 60.0);
        
        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed = Self::normalize_data(data)?;
        self.cache.insert(key.to_string(), processed.clone());
        
        Ok(processed)
    }

    fn normalize_data(data: &[f64]) -> Result<Vec<f64>, String> {
        let max_value = data
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max_value <= 0.0 {
            return Err("Invalid data range for normalization".to_string());
        }

        Ok(data.iter().map(|&x| x / max_value).collect())
    }

    pub fn calculate_statistics(data: &[f64]) -> (f64, f64, f64) {
        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = DataProcessor::normalize_data(&data).unwrap();
        assert_eq!(result, vec![0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let (mean, variance, std_dev) = DataProcessor::calculate_statistics(&data);
        
        assert!((mean - 2.5).abs() < 1e-10);
        assert!((variance - 1.25).abs() < 1e-10);
        assert!((std_dev - 1.118033988749895).abs() < 1e-10);
    }
}