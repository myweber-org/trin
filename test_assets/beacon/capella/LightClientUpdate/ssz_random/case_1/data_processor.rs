
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    InvalidCategory,
    MissingMetadata,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
            ValidationError::MissingMetadata => write!(f, "Required metadata field is missing"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    processed_count: u32,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            processed_count: 0,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ValidationError> {
        Self::validate_record(&record)?;
        self.records.push(record);
        self.processed_count += 1;
        Ok(())
    }

    fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        if record.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        if !["A", "B", "C"].contains(&record.category.as_str()) {
            return Err(ValidationError::InvalidCategory);
        }
        if !record.metadata.contains_key("source") {
            return Err(ValidationError::MissingMetadata);
        }
        Ok(())
    }

    pub fn process_records(&mut self) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        
        for record in &self.records {
            let processed_value = Self::transform_value(record.value);
            results.entry(record.category.clone())
                .and_modify(|v| *v += processed_value)
                .or_insert(processed_value);
        }
        
        results
    }

    fn transform_value(value: f64) -> f64 {
        if value > 100.0 {
            value * 0.9
        } else if value < 10.0 {
            value * 1.1
        } else {
            value
        }
    }

    pub fn get_statistics(&self) -> (u32, f64, f64) {
        let count = self.records.len() as u32;
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        let average = if count > 0 { total / count as f64 } else { 0.0 };
        (count, total, average)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_processed_count(&self) -> u32 {
        self.processed_count
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_valid_record() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 50.0,
            category: "A".to_string(),
            metadata,
        };
        
        assert!(DataProcessor::validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_invalid_id() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 0,
            name: "Test Record".to_string(),
            value: 50.0,
            category: "A".to_string(),
            metadata,
        };
        
        assert!(matches!(
            DataProcessor::validate_record(&record),
            Err(ValidationError::InvalidId)
        ));
    }

    #[test]
    fn test_process_records() {
        let mut processor = DataProcessor::new();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record1 = DataRecord {
            id: 1,
            name: "Record A".to_string(),
            value: 120.0,
            category: "A".to_string(),
            metadata: metadata.clone(),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Record B".to_string(),
            value: 80.0,
            category: "A".to_string(),
            metadata,
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        let results = processor.process_records();
        assert_eq!(results.get("A"), Some(&(108.0 + 80.0)));
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 10.0,
                category: "A".to_string(),
                metadata: metadata.clone(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 20.0,
                category: "B".to_string(),
                metadata: metadata.clone(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 30.0,
                category: "C".to_string(),
                metadata,
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let (count, total, average) = processor.get_statistics();
        assert_eq!(count, 3);
        assert_eq!(total, 60.0);
        assert_eq!(average, 20.0);
    }
}