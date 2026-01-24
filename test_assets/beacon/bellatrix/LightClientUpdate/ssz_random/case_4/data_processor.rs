
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

    pub fn filter_outliers(&mut self, threshold: f64) {
        if let Some(mean) = self.calculate_mean() {
            if let Some(std_dev) = self.calculate_standard_deviation() {
                self.data.retain(|&x| {
                    let z_score = (x - mean).abs() / std_dev;
                    z_score <= threshold
                });
            }
        }
    }

    pub fn get_data(&self) -> &[f64] {
        &self.data
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_data_point(10.0);
        processor.add_data_point(20.0);
        processor.add_data_point(30.0);
        
        assert_eq!(processor.calculate_mean(), Some(20.0));
        assert!(processor.calculate_standard_deviation().unwrap() > 0.0);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n20.3\n30.7\n").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.get_data().len(), 3);
    }

    #[test]
    fn test_outlier_filtering() {
        let mut processor = DataProcessor::new();
        processor.add_data_point(10.0);
        processor.add_data_point(10.2);
        processor.add_data_point(10.1);
        processor.add_data_point(100.0); // Outlier
        
        let initial_len = processor.get_data().len();
        processor.filter_outliers(2.0);
        let filtered_len = processor.get_data().len();
        
        assert!(filtered_len < initial_len);
    }
}