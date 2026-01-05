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
        
        self.data.clear();
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if let Some(value_str) = parts.get(1) {
                if let Ok(value) = value_str.trim().parse::<f64>() {
                    self.data.push(value);
                }
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
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn filter_outliers(&mut self, threshold: f64) {
        if let (Some(mean), Some(std_dev)) = (self.calculate_mean(), self.calculate_standard_deviation()) {
            self.data.retain(|&x| {
                let z_score = (x - mean).abs() / std_dev;
                z_score <= threshold
            });
        }
    }

    pub fn get_summary(&self) -> String {
        let count = self.data.len();
        let mean_str = self.calculate_mean()
            .map(|m| format!("{:.4}", m))
            .unwrap_or_else(|| "N/A".to_string());
        
        let std_dev_str = self.calculate_standard_deviation()
            .map(|s| format!("{:.4}", s))
            .unwrap_or_else(|| "N/A".to_string());
        
        format!("Records: {}, Mean: {}, Std Dev: {}", count, mean_str, std_dev_str)
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
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.calculate_mean(), Some(15.5));
        assert!(processor.calculate_standard_deviation().unwrap() > 0.0);
        
        let summary = processor.get_summary();
        assert!(summary.contains("Records: 3"));
        assert!(summary.contains("Mean: 15.5"));
    }
}