use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_score(&self) -> f64 {
        self.value * 100.0 / (self.id as f64 + 1.0)
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(_) => continue,
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

    pub fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_top_records(&self, n: usize) -> Vec<&DataRecord> {
        let mut sorted_records: Vec<&DataRecord> = self.records.iter().collect();
        sorted_records.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
        sorted_records.into_iter().take(n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string()).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_record() {
        let result = DataRecord::new(1, -5.0, "test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.0,alpha").unwrap();
        writeln!(temp_file, "2,200.0,beta").unwrap();
        writeln!(temp_file, "3,300.0,alpha").unwrap();

        let mut processor = DataProcessor::new();
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.records.len(), 3);
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 100.0, "alpha".to_string()).unwrap());
        processor.records.push(DataRecord::new(2, 200.0, "beta".to_string()).unwrap());
        processor.records.push(DataRecord::new(3, 300.0, "alpha".to_string()).unwrap());

        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 100.0, "test".to_string()).unwrap());
        processor.records.push(DataRecord::new(2, 200.0, "test".to_string()).unwrap());
        processor.records.push(DataRecord::new(3, 300.0, "test".to_string()).unwrap());

        let average = processor.calculate_average_value().unwrap();
        assert_eq!(average, 200.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn summary(&self) -> String {
        format!("ID: {}, Value: {:.2}, Category: {}", self.id, self.value, self.category)
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

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                let id = parts[0].parse::<u32>().unwrap_or(0);
                let value = parts[1].parse::<f64>().unwrap_or(0.0);
                let category = parts[2].trim();

                let record = DataRecord::new(id, value, category);
                self.add_record(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
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

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn valid_count(&self) -> usize {
        self.filter_valid().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_creation() {
        let record = DataRecord::new(1, 42.5, "A");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "A");
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -5.0, "");
        assert!(!record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, 10.0, "A"));
        processor.add_record(DataRecord::new(2, 20.0, "B"));
        processor.add_record(DataRecord::new(3, -5.0, "A"));
        
        assert_eq!(processor.total_records(), 3);
        assert_eq!(processor.valid_count(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), 15.0);
    }

    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value,category")?;
        writeln!(temp_file, "1,10.5,TypeA")?;
        writeln!(temp_file, "2,20.3,TypeB")?;
        writeln!(temp_file, "3,-5.0,TypeA")?;
        
        let mut processor = DataProcessor::new();
        let count = processor.load_from_csv(temp_file.path())?;
        
        assert_eq!(count, 3);
        assert_eq!(processor.total_records(), 3);
        assert_eq!(processor.valid_count(), 2);
        
        Ok(())
    }

    #[test]
    fn test_grouping() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, 10.0, "A"));
        processor.add_record(DataRecord::new(2, 20.0, "B"));
        processor.add_record(DataRecord::new(3, 30.0, "A"));
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("A").unwrap().len(), 2);
        assert_eq!(groups.get("B").unwrap().len(), 1);
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
pub enum DataError {
    InvalidId,
    InvalidValue,
    InvalidCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::InvalidCategory => write!(f, "Category cannot be empty"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    next_id: u32,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_record(&mut self, name: String, value: f64, category: String) -> Result<u32, DataError> {
        if name.trim().is_empty() {
            return Err(DataError::InvalidCategory);
        }
        
        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(DataError::InvalidCategory);
        }

        let id = self.next_id;
        self.next_id += 1;

        let record = DataRecord {
            id,
            name,
            value,
            category,
        };

        self.records.insert(id, record);
        Ok(id)
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn update_record(&mut self, id: u32, value: f64) -> Result<(), DataError> {
        if value < 0.0 || value > 1000.0 {
            return Err(DataError::InvalidValue);
        }

        if let Some(record) = self.records.get_mut(&id) {
            record.value = value;
            Ok(())
        } else {
            Err(DataError::InvalidId)
        }
    }

    pub fn remove_record(&mut self, id: u32) -> Result<DataRecord, DataError> {
        self.records.remove(&id)
            .ok_or(DataError::InvalidId)
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.values()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_all_records(&self) -> Vec<&DataRecord> {
        self.records.values().collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_record() {
        let mut processor = DataProcessor::new();
        let result = processor.add_record("Test".to_string(), 100.0, "A".to_string());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_invalid_value() {
        let mut processor = DataProcessor::new();
        let result = processor.add_record("Test".to_string(), -10.0, "A".to_string());
        assert!(matches!(result, Err(DataError::InvalidValue)));
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor.add_record("A".to_string(), 50.0, "X".to_string()).unwrap();
        processor.add_record("B".to_string(), 100.0, "X".to_string()).unwrap();
        processor.add_record("C".to_string(), 150.0, "Y".to_string()).unwrap();
        
        assert_eq!(processor.calculate_average(), 100.0);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        processor.add_record("A".to_string(), 50.0, "X".to_string()).unwrap();
        processor.add_record("B".to_string(), 100.0, "X".to_string()).unwrap();
        processor.add_record("C".to_string(), 150.0, "Y".to_string()).unwrap();
        
        let filtered = processor.filter_by_category("X");
        assert_eq!(filtered.len(), 2);
    }
}