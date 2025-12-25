use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_numeric_data(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
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

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.data.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.data.iter().sum();
        let mean = sum / self.data.len() as f64;

        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.data.len() as f64;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn process_categorical_data(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let category = line.trim().to_string();
            *self.frequency_map.entry(category).or_insert(0) += 1;
        }
        Ok(())
    }

    pub fn get_top_categories(&self, n: usize) -> Vec<(&String, &u32)> {
        let mut entries: Vec<_> = self.frequency_map.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        entries.into_iter().take(n).collect()
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data
            .iter()
            .filter(|&&x| x >= threshold)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        processor.load_numeric_data(temp_file.path().to_str().unwrap()).unwrap();
        let (mean, variance, std_dev) = processor.calculate_statistics();
        
        assert!((mean - 18.1).abs() < 0.01);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
    }

    #[test]
    fn test_categorical_processing() {
        let mut processor = DataProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "apple\nbanana\napple\norange\nbanana\nbanana").unwrap();
        
        processor.process_categorical_data(temp_file.path().to_str().unwrap()).unwrap();
        let top_categories = processor.get_top_categories(2);
        
        assert_eq!(top_categories.len(), 2);
        assert_eq!(*top_categories[0].0, "banana");
        assert_eq!(*top_categories[0].1, &3);
    }
}