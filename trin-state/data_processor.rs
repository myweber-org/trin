
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        
        if self.values.is_empty() {
            return Err("Empty values vector".into());
        }

        for value in &self.values {
            if !value.is_finite() {
                return Err("Non-finite value detected".into());
            }
        }

        Ok(())
    }
}

pub fn normalize_values(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }

    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    if (max - min).abs() < f64::EPSILON {
        return values.iter().map(|_| 0.5).collect();
    }

    values
        .iter()
        .map(|&v| (v - min) / (max - min))
        .collect()
}

pub fn process_records(records: &mut [DataRecord]) -> Result<(), Box<dyn Error>> {
    for record in records.iter_mut() {
        record.validate()?;
        record.values = normalize_values(&record.values);
        record.add_metadata("processed".to_string(), "true".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = normalize_values(&values);
        
        assert_eq!(normalized.len(), 5);
        assert!((normalized[0] - 0.0).abs() < 0.001);
        assert!((normalized[4] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord::new(1, vec![10.0, 20.0]),
            DataRecord::new(2, vec![30.0, 40.0]),
        ];

        assert!(process_records(&mut records).is_ok());
        assert_eq!(records[0].metadata.get("processed"), Some(&"true".to_string()));
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
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessingError::InvalidId => write!(f, "ID must be greater than zero"),
            ProcessingError::EmptyName => write!(f, "Name cannot be empty"),
            ProcessingError::NegativeValue => write!(f, "Value cannot be negative"),
            ProcessingError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    tag_index: HashMap<String, Vec<u32>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            tag_index: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        
        let id = record.id;
        self.records.insert(id, record.clone());
        
        for tag in &record.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(id);
        }
        
        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.id == 0 {
            return Err(ProcessingError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ProcessingError::EmptyName);
        }
        
        if record.value < 0.0 {
            return Err(ProcessingError::NegativeValue);
        }
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &record.tags {
            if !seen_tags.insert(tag) {
                return Err(ProcessingError::DuplicateTag);
            }
        }
        
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.tag_index
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.records.get(id))
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let total: f64 = self.records.values().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
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
            name: "Test".to_string(),
            value: 100.0,
            tags: vec!["important".to_string(), "test".to_string()],
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_find_by_tag() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 50.0,
            tags: vec!["urgent".to_string()],
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Second".to_string(),
            value: 75.0,
            tags: vec!["urgent".to_string(), "review".to_string()],
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        let urgent_records = processor.find_by_tag("urgent");
        assert_eq!(urgent_records.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "A".to_string(),
                value: 10.0,
                tags: vec![],
            },
            DataRecord {
                id: 2,
                name: "B".to_string(),
                value: 20.0,
                tags: vec![],
            },
            DataRecord {
                id: 3,
                name: "C".to_string(),
                value: 30.0,
                tags: vec![],
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.calculate_average(), 20.0);
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

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }
    
    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}