
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err(format!("Value cannot be negative: {}", value));
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    
    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()
                .map_err(|e| format!("Invalid ID at line {}: {}", line_num + 1, e))?;
            
            let value = parts[1].parse::<f64>()
                .map_err(|e| format!("Invalid value at line {}: {}", line_num + 1, e))?;
            
            let record = DataRecord::new(id, value, parts[2])?;
            self.records.push(record);
        }
        
        Ok(())
    }
    
    pub fn calculate_statistics(&self) -> Statistics {
        if self.records.is_empty() {
            return Statistics::default();
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len();
        let mean = sum / count as f64;
        
        let min = self.records.iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);
        
        let max = self.records.iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);
        
        let variance: f64 = self.records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        Statistics {
            count,
            sum,
            mean,
            min,
            max,
            variance,
        }
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.iter()
            .filter(|r| r.category == category)
            .collect()
    }
    
    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }
    
    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub variance: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Statistics: count={}, sum={:.2}, mean={:.2}, min={:.2}, max={:.2}, variance={:.2}",
               self.count, self.sum, self.mean, self.min, self.max, self.variance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_data_record() {
        let result = DataRecord::new(1, -5.0, "test");
        assert!(result.is_err());
        
        let result = DataRecord::new(1, 5.0, "");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,42.5,category_a")?;
        writeln!(temp_file, "2,18.3,category_b")?;
        writeln!(temp_file, "# This is a comment")?;
        writeln!(temp_file, "3,77.1,category_a")?;
        
        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path())?;
        
        assert_eq!(processor.get_records().len(), 3);
        assert_eq!(processor.filter_by_category("category_a").len(), 2);
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats.count, 3);
        assert!((stats.mean - 45.966666).abs() < 0.001);
        
        Ok(())
    }
    
    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        let stats = processor.calculate_statistics();
        assert_eq!(stats, Statistics::default());
    }
}use std::error::Error;
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

    pub fn find_min_max(&self) -> Option<(f64, f64)> {
        if self.data.is_empty() {
            return None;
        }
        
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Some((min, max))
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
    }

    pub fn clear_data(&mut self) {
        self.data.clear();
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
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 5);
        
        let mean = processor.calculate_mean();
        assert!(mean.is_some());
        
        let std_dev = processor.calculate_standard_deviation();
        assert!(std_dev.is_some());
        
        let min_max = processor.find_min_max();
        assert_eq!(min_max, Some((10.5, 25.1)));
        
        processor.clear_data();
        assert_eq!(processor.data_count(), 0);
    }
}