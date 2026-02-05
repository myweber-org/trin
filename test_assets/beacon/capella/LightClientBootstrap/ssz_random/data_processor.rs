use csv::Reader;
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

fn process_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut records: Vec<Record> = Vec::new();
    
    for result in reader.deserialize() {
        let record: Record = result?;
        if record.active && record.value > 0.0 {
            records.push(record);
        }
    }
    
    let output_file = File::create(output_path)?;
    let mut writer = csv::Writer::from_writer(output_file);
    
    for record in records {
        writer.serialize(record)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: String::from("A"), value: 10.0, active: true },
            Record { id: 2, name: String::from("B"), value: 20.0, active: true },
            Record { id: 3, name: String::from("C"), value: 30.0, active: true },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
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

    pub fn process_numeric_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed = Self::normalize_data(values);
        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    fn normalize_data(values: &[f64]) -> Vec<f64> {
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;

        if range.abs() < f64::EPSILON {
            return vec![0.0; values.len()];
        }

        values
            .iter()
            .map(|&v| (v - min) / range)
            .collect()
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|values| {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
            let std_dev = variance.sqrt();
            (mean, variance, std_dev)
        })
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
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = DataProcessor::normalize_data(&data);
        assert_eq!(normalized, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn test_empty_data_error() {
        let mut processor = DataProcessor::new();
        let result = processor.process_numeric_data("test", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let first_result = processor.process_numeric_data("dataset1", &data).unwrap();
        let second_result = processor.process_numeric_data("dataset1", &data).unwrap();
        
        assert_eq!(first_result, second_result);
    }
}