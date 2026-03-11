
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

        let processed = Self::normalize_data(data)?;
        let transformed = Self::apply_transformations(&processed);
        
        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn normalize_data(data: &[f64]) -> Result<Vec<f64>, String> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        if variance.abs() < 1e-10 {
            return Err("Zero variance detected".to_string());
        }

        let std_dev = variance.sqrt();
        Ok(data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect())
    }

    fn apply_transformations(data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).sin())
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
    fn test_normalization() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = DataProcessor::normalize_data(&data).unwrap();
        
        let mean = result.iter().sum::<f64>() / result.len() as f64;
        let variance = result.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / result.len() as f64;
        
        assert!(mean.abs() < 1e-10);
        assert!((variance - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter: ',',
        }
    }

    pub fn set_delimiter(&mut self, delimiter: char) -> &mut Self {
        self.delimiter = delimiter;
        self
    }

    pub fn process_file<F>(&self, filter_fn: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line_content = line?;
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if filter_fn(&fields) {
                results.push(fields);
            }
        }

        Ok(results)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines().count())
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
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,item_a,100").unwrap();
        writeln!(temp_file, "2,item_b,200").unwrap();
        writeln!(temp_file, "3,item_c,300").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor
            .process_file(|fields| fields[2].parse::<i32>().unwrap_or(0) > 150)
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0][1], "item_b");
        assert_eq!(result[1][1], "item_c");
    }

    #[test]
    fn test_record_count() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "header1,header2").unwrap();
        writeln!(temp_file, "data1,data2").unwrap();
        writeln!(temp_file, "data3,data4").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let count = processor.count_records().unwrap();
        assert_eq!(count, 3);
    }
}