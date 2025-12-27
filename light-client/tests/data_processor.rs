
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
    MissingField,
    CategoryNotFound,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::MissingField => write!(f, "Required field is missing"),
            DataError::CategoryNotFound => write!(f, "Category does not exist"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_map: HashMap<String, Vec<u32>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_map: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.value < 0.0 || record.value > 10000.0 {
            return Err(DataError::InvalidValue);
        }
        
        if record.name.trim().is_empty() || record.category.trim().is_empty() {
            return Err(DataError::MissingField);
        }

        self.records.push(record.clone());
        self.category_map
            .entry(record.category.clone())
            .or_insert_with(Vec::new)
            .push(record.id);

        Ok(())
    }

    pub fn get_records_by_category(&self, category: &str) -> Result<Vec<&DataRecord>, DataError> {
        let ids = self.category_map.get(category)
            .ok_or(DataError::CategoryNotFound)?;
        
        let mut result = Vec::new();
        for id in ids {
            if let Some(record) = self.records.iter().find(|r| r.id == *id) {
                result.push(record);
            }
        }
        
        Ok(result)
    }

    pub fn calculate_average(&self, category: &str) -> Result<f64, DataError> {
        let records = self.get_records_by_category(category)?;
        
        if records.is_empty() {
            return Ok(0.0);
        }
        
        let sum: f64 = records.iter().map(|r| r.value).sum();
        Ok(sum / records.len() as f64)
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
        
        self.rebuild_category_map();
    }

    fn rebuild_category_map(&mut self) {
        self.category_map.clear();
        for record in &self.records {
            self.category_map
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.id);
        }
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_categories(&self) -> Vec<&String> {
        self.category_map.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 42.5,
            category: "Test".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.total_records(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Invalid".to_string(),
            value: 50.0,
            category: "Test".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord { id: 1, name: "A".to_string(), value: 10.0, category: "Cat1".to_string() },
            DataRecord { id: 2, name: "B".to_string(), value: 20.0, category: "Cat1".to_string() },
            DataRecord { id: 3, name: "C".to_string(), value: 30.0, category: "Cat2".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let avg = processor.calculate_average("Cat1").unwrap();
        assert_eq!(avg, 15.0);
    }
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
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

    pub fn load_from_csv(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            record.validate()?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn save_to_csv(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = Writer::from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: Record) -> Result<(), String> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records
            .iter()
            .map(|r| r.value)
            .sum()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&Record> {
        self.records
            .iter()
            .find(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record::new(2, "".to_string(), -50.0, false);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = Record::new(1, "Item1".to_string(), 10.5, true);
        let record2 = Record::new(2, "Item2".to_string(), 20.0, false);
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        assert_eq!(processor.filter_active().len(), 1);
        assert_eq!(processor.calculate_total(), 30.5);
        assert!(processor.find_by_id(1).is_some());
        assert!(processor.find_by_id(3).is_none());
    }
}