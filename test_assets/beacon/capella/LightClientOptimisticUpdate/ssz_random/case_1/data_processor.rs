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

            let value: f64 = line.trim().parse()?;
            self.data.push(value);
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
            "Data points: {}, Mean: {}, Standard Deviation: {}",
            self.data.len(),
            mean_str,
            std_dev_str
        )
    }

    pub fn filter_outliers(&self, threshold: f64) -> Vec<f64> {
        let mean = match self.calculate_mean() {
            Some(m) => m,
            None => return Vec::new(),
        };

        let std_dev = match self.calculate_standard_deviation() {
            Some(s) => s,
            None => return Vec::new(),
        };

        self.data
            .iter()
            .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        
        let mean = processor.calculate_mean();
        assert!(mean.is_some());
        
        let std_dev = processor.calculate_standard_deviation();
        assert!(std_dev.is_some());
        
        let filtered = processor.filter_outliers(2.0);
        assert!(!filtered.is_empty());
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut categories = std::collections::HashMap::new();

        for record in &self.records {
            categories
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }

        categories
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, usize) {
        let total = self.records.len();
        let average = self.calculate_average();
        let valid_count = self.validate_records().len();

        (total, average, valid_count)
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,15.0,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,-5.0,Category1").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let (total, average, valid_count) = processor.get_statistics();
        assert_eq!(total, 3);
        assert_eq!(valid_count, 2);
        assert!((average.unwrap() - 12.75).abs() < 0.001);

        let categories = processor.group_by_category();
        assert_eq!(categories.get("Category1").unwrap().len(), 2);
        assert_eq!(categories.get("Category2").unwrap().len(), 1);
    }
}