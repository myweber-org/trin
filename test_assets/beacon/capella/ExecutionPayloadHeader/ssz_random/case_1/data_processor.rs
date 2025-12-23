
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

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|f| !f.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        if data.is_empty() {
            return Err("No data available for statistics".into());
        }

        let mut values = Vec::new();
        for record in data {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }

            match record[column_index].parse::<f64>() {
                Ok(value) => values.push(value),
                Err(_) => return Err(format!("Invalid numeric value at column {}", column_index).into()),
            }
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();

        Ok((mean, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,87.2").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![
            vec!["10.0".to_string(), "20.0".to_string()],
            vec!["20.0".to_string(), "30.0".to_string()],
            vec!["30.0".to_string(), "40.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let (mean, std_dev) = processor.calculate_statistics(&data, 0).unwrap();
        
        assert!((mean - 20.0).abs() < 0.001);
        assert!((std_dev - 8.1649).abs() < 0.001);
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
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.category.is_empty())
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

    pub fn find_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
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
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,42.5,TypeA").unwrap();
        writeln!(temp_file, "2,18.3,TypeB").unwrap();
        writeln!(temp_file, "3,-5.0,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 30.4).abs() < 0.001);
        
        let type_a_records = processor.find_by_category("TypeA");
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
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = DataProcessor::normalize_data(&data);
        
        let sum: f64 = normalized.iter().sum();
        let sum_sq: f64 = normalized.iter().map(|x| x.powi(2)).sum();
        
        assert!(sum.abs() < 1e-10);
        assert!((sum_sq - (data.len() as f64 - 1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Empty dataset provided");
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let first_result = processor.process_dataset("key1", &data).unwrap();
        let second_result = processor.process_dataset("key1", &data).unwrap();
        
        assert_eq!(first_result, second_result);
        
        let (unique_keys, total_items) = processor.get_cache_stats();
        assert_eq!(unique_keys, 1);
        assert_eq!(total_items, 3);
    }
}