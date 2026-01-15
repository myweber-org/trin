
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

        let processed = Self::normalize_data(data);
        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    fn normalize_data(data: &[f64]) -> Vec<f64> {
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

    pub fn calculate_statistics(&self, data: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if data.is_empty() {
            return stats;
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data.iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        let sorted_data = {
            let mut sorted = data.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted
        };
        
        let median = if data.len() % 2 == 0 {
            (sorted_data[data.len() / 2 - 1] + sorted_data[data.len() / 2]) / 2.0
        } else {
            sorted_data[data.len() / 2]
        };

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("median".to_string(), median);
        stats.insert("min".to_string(), *sorted_data.first().unwrap());
        stats.insert("max".to_string(), *sorted_data.last().unwrap());

        stats
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = DataProcessor::normalize_data(&data);
        
        assert_eq!(normalized[0], 0.0);
        assert_eq!(normalized[4], 1.0);
        assert!(normalized[2] > 0.4 && normalized[2] < 0.6);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty dataset provided");
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let result1 = processor.process_dataset("dataset1", &data).unwrap();
        let result2 = processor.process_dataset("dataset1", &data).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(processor.cache_size(), 1);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let stats = processor.calculate_statistics(&data);
        
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("median").unwrap(), &3.0);
        assert_eq!(stats.get("min").unwrap(), &1.0);
        assert_eq!(stats.get("max").unwrap(), &5.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.validate_record(&record)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), String> {
        if record.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if record.value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if record.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,20.3,TypeB").unwrap();
        writeln!(temp_file, "3,15.7,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.001);
        
        let type_a_records = processor.filter_by_category("TypeA");
        assert_eq!(type_a_records.len(), 2);
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

    pub fn process_numeric_data(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data array provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated_data: Vec<f64> = data
            .iter()
            .filter(|&&x| x.is_finite())
            .cloned()
            .collect();

        if validated_data.len() < data.len() {
            return Err("Invalid numeric values detected in input data".to_string());
        }

        let mean = validated_data.iter().sum::<f64>() / validated_data.len() as f64;
        let processed: Vec<f64> = validated_data
            .iter()
            .map(|&x| (x - mean).abs())
            .collect();

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_entries = self.cache.len();
        let total_values: usize = self.cache.values().map(|v| v.len()).sum();
        (total_entries, total_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_valid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = processor.process_numeric_data("test_key", &data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
    }

    #[test]
    fn test_process_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, f64::NAN, 3.0];
        let result = processor.process_numeric_data("invalid_key", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let first_result = processor.process_numeric_data("cached_key", &data).unwrap();
        let second_result = processor.process_numeric_data("cached_key", &data).unwrap();
        
        assert_eq!(first_result, second_result);
        assert_eq!(processor.get_cache_stats().0, 1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let invalid_record = vec!["".to_string(), "value".to_string()];
        
        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_extract_column() {
        let processor = DataProcessor::new(',', false);
        let data = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let column = processor.extract_column(&data, 1);
        assert_eq!(column, vec!["b".to_string(), "e".to_string()]);
    }
}