
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: String) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
    }
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
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();
            let timestamp = parts[3].to_string();

            let record = DataRecord::new(id, value, category, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|record| record.value).collect();
        
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let avg = sum / values.len() as f64;

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test".to_string(), "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, 42.5, "test".to_string(), "2024-01-01".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,100.5,type_a,2024-01-01").unwrap();
        writeln!(temp_file, "2,200.3,type_b,2024-01-02").unwrap();
        writeln!(temp_file, "3,300.7,type_a,2024-01-03").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 100.0, "A".to_string(), "ts1".to_string()));
        processor.records.push(DataRecord::new(2, 200.0, "B".to_string(), "ts2".to_string()));
        processor.records.push(DataRecord::new(3, 300.0, "A".to_string(), "ts3".to_string()));

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test".to_string(), "ts1".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "test".to_string(), "ts2".to_string()));
        processor.records.push(DataRecord::new(3, 30.0, "test".to_string(), "ts3".to_string()));

        let (min, max, avg) = processor.get_statistics();
        assert_eq!(min, 10.0);
        assert_eq!(max, 30.0);
        assert_eq!(avg, 20.0);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    InvalidCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let valid_categories = ["A", "B", "C"];
        if !valid_categories.contains(&self.category.as_str()) {
            return Err(ValidationError::InvalidCategory);
        }
        
        Ok(())
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_totals: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_totals: HashMap::new(),
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        record.validate()?;
        
        self.records.push(record.clone());
        
        let total = self.category_totals
            .entry(record.category.clone())
            .or_insert(0.0);
        *total += record.value;
        
        Ok(())
    }
    
    pub fn get_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }
    
    pub fn get_category_total(&self, category: &str) -> Option<f64> {
        self.category_totals.get(category).copied()
    }
    
    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.get_total_value() / self.records.len() as f64)
        }
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
    
    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
        
        self.recalculate_totals();
    }
    
    fn recalculate_totals(&mut self) {
        self.category_totals.clear();
        for record in &self.records {
            let total = self.category_totals
                .entry(record.category.clone())
                .or_insert(0.0);
            *total += record.value;
        }
    }
    
    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Item1".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Item2".to_string(),
            value: 75.0,
            category: "B".to_string(),
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        assert_eq!(processor.get_record_count(), 2);
        assert_eq!(processor.get_total_value(), 125.0);
        assert_eq!(processor.get_category_total("A"), Some(50.0));
        assert_eq!(processor.get_average_value(), Some(62.5));
    }
    
    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "A".to_string(),
        };
        
        processor.add_record(record).unwrap();
        
        processor.transform_values(|x| x * 2.0);
        
        assert_eq!(processor.get_total_value(), 20.0);
    }
}