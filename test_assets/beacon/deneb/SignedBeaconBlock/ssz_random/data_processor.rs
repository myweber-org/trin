
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
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

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn get_data_count(&self) -> usize {
        self.data.len()
    }

    pub fn add_data_point(&mut self, value: f64) {
        self.data.push(value);
    }

    pub fn clear_data(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.get_data_count(), 0);
        assert_eq!(processor.calculate_mean(), None);
        assert_eq!(processor.calculate_standard_deviation(), None);
    }

    #[test]
    fn test_basic_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_data_point(10.0);
        processor.add_data_point(20.0);
        processor.add_data_point(30.0);
        
        assert_eq!(processor.get_data_count(), 3);
        assert_eq!(processor.calculate_mean(), Some(20.0));
        assert!(processor.calculate_standard_deviation().unwrap() > 0.0);
    }

    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "10.5")?;
        writeln!(temp_file, "20.3")?;
        writeln!(temp_file, "15.7")?;
        
        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path())?;
        
        assert_eq!(processor.get_data_count(), 3);
        assert!(processor.calculate_mean().unwrap() > 0.0);
        
        Ok(())
    }

    #[test]
    fn test_data_clearing() {
        let mut processor = DataProcessor::new();
        processor.add_data_point(5.0);
        processor.add_data_point(15.0);
        
        assert_eq!(processor.get_data_count(), 2);
        
        processor.clear_data();
        assert_eq!(processor.get_data_count(), 0);
        assert_eq!(processor.calculate_mean(), None);
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

    pub fn process_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }

        let processed: Vec<f64> = values
            .iter()
            .map(|&x| x * 2.0)
            .filter(|&x| x > 0.0)
            .collect();

        if processed.is_empty() {
            return Err("All values filtered out".to_string());
        }

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn get_cached_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            (mean, variance, std_dev)
        })
    }
}

pub fn validate_input(input: &str) -> Result<Vec<f64>, String> {
    let numbers: Result<Vec<f64>, _> = input
        .split(',')
        .map(|s| s.trim().parse::<f64>())
        .collect();

    match numbers {
        Ok(nums) if !nums.is_empty() => Ok(nums),
        Ok(_) => Err("No valid numbers found".to_string()),
        Err(_) => Err("Failed to parse input as numbers".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_data_valid() {
        let mut processor = DataProcessor::new();
        let result = processor.process_data("test", &[1.0, 2.0, 3.0]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_process_data_invalid() {
        let mut processor = DataProcessor::new();
        let result = processor.process_data("test", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut processor = DataProcessor::new();
        processor.process_data("stats", &[1.0, 2.0, 3.0]).unwrap();
        let stats = processor.calculate_statistics("stats");
        assert!(stats.is_some());
        
        let (mean, variance, std_dev) = stats.unwrap();
        assert_eq!(mean, 4.0);
        assert_eq!(variance, 8.0);
        assert_eq!(std_dev, 2.8284271247461903);
    }
}