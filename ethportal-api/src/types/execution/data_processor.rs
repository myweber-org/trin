use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let records = self.process()?;
        let filtered: Vec<Vec<String>> = records
            .into_iter()
            .filter(|record| predicate(record))
            .collect();
        
        Ok(filtered)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let records = self.process()?;
        Ok(records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let records = processor.process().unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["name", "age", "city"]);
        assert_eq!(records[1], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_filter_records() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let filtered = processor.filter_records(|record| {
            record.len() > 1 && record[1].parse::<i32>().unwrap_or(0) > 30
        }).unwrap();
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["Charlie", "35", "Paris"]);
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        let mut reader = Reader::from_path(file_path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn max_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).reduce(f64::max)
    }

    pub fn min_value(&self) -> Option<f64> {
        self.records.iter().map(|r| r.value).reduce(f64::min)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,value,category").unwrap();
        writeln!(file, "1,10.5,A").unwrap();
        writeln!(file, "2,20.3,B").unwrap();
        writeln!(file, "3,15.7,A").unwrap();
        writeln!(file, "4,25.1,B").unwrap();
        file
    }

    #[test]
    fn test_load_and_calculate() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        processor.load_csv(test_file.path()).unwrap();

        assert_eq!(processor.record_count(), 4);
        assert_eq!(processor.calculate_mean(), Some(17.9));
        assert_eq!(processor.max_value(), Some(25.1));
        assert_eq!(processor.min_value(), Some(10.5));
    }

    #[test]
    fn test_filtering() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        processor.load_csv(test_file.path()).unwrap();

        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);

        let category_b = processor.filter_by_category("B");
        assert_eq!(category_b.len(), 2);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }

        Ok(DataProcessor { threshold })
    }

    pub fn process_data(&self, input: Vec<f64>) -> Result<Vec<f64>, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }

        let filtered_data: Vec<f64> = input
            .into_iter()
            .filter(|&value| value >= self.threshold)
            .collect();

        if filtered_data.is_empty() {
            return Err(ValidationError {
                message: format!(
                    "No data points meet the threshold requirement of {}",
                    self.threshold
                ),
            });
        }

        Ok(filtered_data)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> (f64, f64, f64) {
        let count = data.len() as f64;
        let sum: f64 = data.iter().sum();
        let mean = sum / count;

        let variance: f64 = data
            .iter()
            .map(|value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        if data.is_empty() {
            return Vec::new();
        }

        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;

        if range == 0.0 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&value| (value - min) / range)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_data() {
        let processor = DataProcessor::new(0.3).unwrap();
        let input = vec![0.1, 0.4, 0.2, 0.5, 0.6];
        let result = processor.process_data(input).unwrap();
        assert_eq!(result, vec![0.4, 0.5, 0.6]);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&data);
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_data() {
        let processor = DataProcessor::new(0.0).unwrap();
        let data = vec![10.0, 20.0, 30.0, 40.0];
        let normalized = processor.normalize_data(&data);
        assert_eq!(normalized, vec![0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0]);
    }
}