
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStatistics>,
}

#[derive(Debug, Clone)]
pub struct CategoryStatistics {
    pub category: String,
    pub count: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        self.records.push(record.clone());
        self.update_statistics(&record);
        Ok(())
    }

    pub fn process_records(&mut self) -> Result<Vec<TransformedRecord>, ProcessingError> {
        let mut transformed = Vec::new();
        
        for record in &self.records {
            let transformed_record = self.transform_record(record)?;
            transformed.push(transformed_record);
        }
        
        Ok(transformed)
    }

    pub fn get_statistics(&self) -> &HashMap<String, CategoryStatistics> {
        &self.category_stats
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string()
            ));
        }
        
        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError(
                "Record value cannot be negative".to_string()
            ));
        }
        
        if record.category.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record category cannot be empty".to_string()
            ));
        }
        
        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> Result<TransformedRecord, ProcessingError> {
        let normalized_value = if record.value > 1000.0 {
            record.value / 1000.0
        } else {
            record.value
        };
        
        let processed_name = record.name.to_uppercase();
        
        let status = if normalized_value > 50.0 {
            "HIGH"
        } else if normalized_value > 10.0 {
            "MEDIUM"
        } else {
            "LOW"
        }.to_string();

        Ok(TransformedRecord {
            original_id: record.id,
            processed_name,
            normalized_value,
            category: record.category.clone(),
            status,
        })
    }

    fn update_statistics(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStatistics {
                category: record.category.clone(),
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });
        
        stats.count += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.count as f64;
    }
}

#[derive(Debug, Clone)]
pub struct TransformedRecord {
    pub original_id: u32,
    pub processed_name: String,
    pub normalized_value: f64,
    pub category: String,
    pub status: String,
}

impl DataRecord {
    pub fn new(id: u32, name: &str, value: f64, category: &str) -> Self {
        DataRecord {
            id,
            name: name.to_string(),
            value,
            category: category.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor_validation() {
        let processor = DataProcessor::new();
        let valid_record = DataRecord::new(1, "Test", 100.0, "CategoryA");
        let invalid_record = DataRecord::new(2, "", -10.0, "");
        
        assert!(processor.validate_record(&valid_record).is_ok());
        assert!(processor.validate_record(&invalid_record).is_err());
    }

    #[test]
    fn test_add_and_process_records() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, "Item1", 1500.0, "Electronics");
        let record2 = DataRecord::new(2, "Item2", 25.0, "Books");
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        let transformed = processor.process_records();
        assert!(transformed.is_ok());
        assert_eq!(transformed.unwrap().len(), 2);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.len(), 2);
        assert!(stats.contains_key("Electronics"));
        assert!(stats.contains_key("Books"));
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, "Item1", 100.0, "Electronics")).unwrap();
        processor.add_record(DataRecord::new(2, "Item2", 200.0, "Books")).unwrap();
        processor.add_record(DataRecord::new(3, "Item3", 300.0, "Electronics")).unwrap();
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let books = processor.filter_by_category("Books");
        assert_eq!(books.len(), 1);
    }
}