
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
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub total_value: f64,
    pub record_count: usize,
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
        self.update_category_stats(&record);
        Ok(())
    }

    pub fn process_records(&mut self) -> Result<(), ProcessingError> {
        if self.records.is_empty() {
            return Err(ProcessingError::InvalidData("No records to process".to_string()));
        }

        self.calculate_category_stats();
        self.normalize_values()?;
        Ok(())
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn get_all_stats(&self) -> &HashMap<String, CategoryStats> {
        &self.category_stats
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.is_empty() {
            return Err(ProcessingError::ValidationError("Record name cannot be empty".to_string()));
        }
        
        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError("Record value cannot be negative".to_string()));
        }
        
        if record.category.is_empty() {
            return Err(ProcessingError::ValidationError("Record category cannot be empty".to_string()));
        }
        
        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                total_value: 0.0,
                record_count: 0,
                average_value: 0.0,
            });
        
        stats.total_value += record.value;
        stats.record_count += 1;
        stats.average_value = stats.total_value / stats.record_count as f64;
    }

    fn calculate_category_stats(&mut self) {
        self.category_stats.clear();
        for record in &self.records {
            self.update_category_stats(record);
        }
    }

    fn normalize_values(&mut self) -> Result<(), ProcessingError> {
        let total = self.calculate_total_value();
        if total == 0.0 {
            return Err(ProcessingError::TransformationFailed("Total value is zero, cannot normalize".to_string()));
        }

        for record in &mut self.records {
            record.value = (record.value / total) * 100.0;
        }
        
        self.calculate_category_stats();
        Ok(())
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
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 50.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 100.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 75.0,
                category: "CategoryB".to_string(),
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        processor.process_records().unwrap();
        
        let stats_a = processor.get_category_stats("CategoryA").unwrap();
        assert_eq!(stats_a.total_value, 150.0);
        assert_eq!(stats_a.record_count, 2);
        assert_eq!(stats_a.average_value, 75.0);
        
        let stats_b = processor.get_category_stats("CategoryB").unwrap();
        assert_eq!(stats_b.total_value, 75.0);
        assert_eq!(stats_b.record_count, 1);
        assert_eq!(stats_b.average_value, 75.0);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 50.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 100.0,
                category: "CategoryB".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 75.0,
                category: "CategoryA".to_string(),
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let category_a_records = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_records.len(), 2);
        
        let category_b_records = processor.filter_by_category("CategoryB");
        assert_eq!(category_b_records.len(), 1);
    }
}