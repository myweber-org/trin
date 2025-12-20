use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
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

    pub fn get_summary(&self) -> String {
        let mean = self.calculate_mean()
            .map_or("N/A".to_string(), |v| format!("{:.4}", v));
        let std_dev = self.calculate_standard_deviation()
            .map_or("N/A".to_string(), |v| format!("{:.4}", v));
        let count = self.data.len();

        format!("Count: {}, Mean: {}, Std Dev: {}", count, mean, std_dev)
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
        writeln!(temp_file, "10.5\n15.2\n12.8\n18.3\n14.1").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let mean = processor.calculate_mean().unwrap();
        let std_dev = processor.calculate_standard_deviation().unwrap();
        
        assert!((mean - 14.18).abs() < 0.01);
        assert!((std_dev - 2.89).abs() < 0.01);
    }
}