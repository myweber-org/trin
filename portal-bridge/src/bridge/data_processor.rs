use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, &'static str> {
        if value < 0.0 {
            return Err("Value cannot be negative");
        }
        if category.is_empty() {
            return Err("Category cannot be empty");
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let raw_record: (u32, f64, String) = result?;
        match DataRecord::new(raw_record.0, raw_record.1, raw_record.2) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Skipping invalid record: {}", e),
        }
    }

    Ok(records)
}

pub fn process_records(records: &[DataRecord]) -> (f64, usize) {
    let total: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    (total, count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 42.5, "alpha".to_string());
        assert!(record.is_ok());
        assert_eq!(record.unwrap().id, 1);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "beta".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_calculation() {
        let record = DataRecord::new(3, 10.0, "gamma".to_string()).unwrap();
        assert_eq!(record.calculate_adjusted_value(2.5), 25.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            // Skip header and empty lines
            if line_num == 0 || line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();
            let valid = parts[3].parse::<bool>().unwrap_or(false);
            
            self.records.push(DataRecord {
                id,
                value,
                category,
                valid,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_valid_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.valid)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.filter_valid_records();
        if valid_records.is_empty() {
            return None;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    pub fn get_statistics(&self) -> (usize, usize, Option<f64>) {
        let total = self.records.len();
        let valid_count = self.filter_valid_records().len();
        let avg = self.calculate_average();
        
        (total, valid_count, avg)
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
        
        // Create test CSV
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,TypeA,true").unwrap();
        writeln!(temp_file, "2,20.3,TypeB,false").unwrap();
        writeln!(temp_file, "3,15.7,TypeA,true").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        
        // Test loading
        let count = processor.load_from_csv(file_path).unwrap();
        assert_eq!(count, 3);
        
        // Test statistics
        let (total, valid, avg) = processor.get_statistics();
        assert_eq!(total, 3);
        assert_eq!(valid, 2);
        assert_eq!(avg, Some(13.1));
        
        // Test grouping
        let groups = processor.group_by_category();
        assert_eq!(groups.get("TypeA").unwrap().len(), 2);
        assert_eq!(groups.get("TypeB").unwrap().len(), 1);
    }
}
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

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            for part in parts {
                if let Ok(value) = part.trim().parse::<f64>() {
                    self.data.push(value);
                } else {
                    self.frequency_map
                        .entry(part.trim().to_string())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
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
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / self.data.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn get_top_categories(&self, limit: usize) -> Vec<(String, u32)> {
        let mut entries: Vec<_> = self.frequency_map.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        
        entries
            .iter()
            .take(limit)
            .map(|(key, value)| (key.clone(), *value))
            .collect()
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data
            .iter()
            .filter(|&&value| value >= threshold)
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
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "12.5,15.3,category_a").unwrap();
        writeln!(temp_file, "8.7,category_b,20.1").unwrap();
        writeln!(temp_file, "category_a,18.9,14.2").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let stats = processor.calculate_statistics();
        assert!(stats.0 > 0.0);
        
        let top_categories = processor.get_top_categories(2);
        assert_eq!(top_categories.len(), 2);
        
        let filtered = processor.filter_data(10.0);
        assert!(!filtered.is_empty());
    }
}