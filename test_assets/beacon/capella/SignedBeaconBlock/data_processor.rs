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
            if line.trim().is_empty() {
                continue;
            }

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
        let mean_str = match self.calculate_mean() {
            Some(mean) => format!("{:.4}", mean),
            None => "N/A".to_string(),
        };

        let std_dev_str = match self.calculate_standard_deviation() {
            Some(std_dev) => format!("{:.4}", std_dev),
            None => "N/A".to_string(),
        };

        format!(
            "Data Summary:\n  Count: {}\n  Mean: {}\n  Standard Deviation: {}",
            self.data.len(),
            mean_str,
            std_dev_str
        )
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data
            .iter()
            .filter(|&&x| x > threshold)
            .copied()
            .collect()
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
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();

        processor.load_from_csv(temp_file.path()).unwrap();

        assert_eq!(processor.data.len(), 5);
        assert!(processor.calculate_mean().unwrap() - 18.1 < 0.0001);
        assert!(processor.calculate_standard_deviation().unwrap() - 5.5 < 0.1);

        let filtered = processor.filter_by_threshold(18.0);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_empty_data() {
        let processor = DataProcessor::new();
        assert!(processor.calculate_mean().is_none());
        assert!(processor.calculate_standard_deviation().is_none());
    }
}