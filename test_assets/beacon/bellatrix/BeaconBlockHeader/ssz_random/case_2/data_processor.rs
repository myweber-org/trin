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

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let mut result = Vec::with_capacity(data.len());
        
        for &value in data {
            if !value.is_finite() {
                return Err(format!("Invalid numeric value detected: {}", value));
            }
            if value < 0.0 {
                return Err("Negative values not allowed".to_string());
            }
            result.push(value);
        }
        
        Ok(result)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        let max_value = data.iter().fold(f64::MIN, |a, &b| a.max(b));
        if max_value == 0.0 {
            return vec![0.0; data.len()];
        }
        
        data.iter()
            .map(|&x| x / max_value)
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| (x * 100.0).ln_1p())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0];
        
        let result = processor.process_dataset("test", &data).unwrap();
        assert_eq!(result.len(), 4);
        
        let cached_result = processor.process_dataset("test", &data).unwrap();
        assert_eq!(result, cached_result);
    }

    #[test]
    fn test_validation_rejects_negative() {
        let processor = DataProcessor::new();
        let data = vec![1.0, -2.0, 3.0];
        
        assert!(processor.validate_data(&data).is_err());
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_data(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

pub fn calculate_statistics(path: &str) -> Result<(f64, f64, usize), Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut sum = 0.0;
    let mut count = 0;
    let mut max_value = f64::MIN;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.active {
            sum += record.value;
            count += 1;
            if record.value > max_value {
                max_value = record.value;
            }
        }
    }

    let average = if count > 0 { sum / count as f64 } else { 0.0 };
    Ok((average, max_value, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_data() {
        let input_data = "id,name,value,active\n1,test1,10.5,true\n2,test2,5.0,false\n3,test3,15.0,true\n";
        let input_file = NamedTempFile::new().unwrap();
        std::fs::write(input_file.path(), input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        process_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            10.0
        ).unwrap();
        
        let output = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(output.contains("test1"));
        assert!(!output.contains("test2"));
        assert!(output.contains("test3"));
    }
}