
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
    InvalidValue,
    MissingField,
    CategoryNotFound,
    TransformationFailed,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue => write!(f, "Invalid numeric value"),
            ProcessingError::MissingField => write!(f, "Required field is missing"),
            ProcessingError::CategoryNotFound => write!(f, "Category not found in mapping"),
            ProcessingError::TransformationFailed => write!(f, "Data transformation failed"),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    category_mapping: HashMap<String, String>,
    validation_threshold: f64,
}

impl DataProcessor {
    pub fn new(category_mapping: HashMap<String, String>, validation_threshold: f64) -> Self {
        DataProcessor {
            category_mapping,
            validation_threshold,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value < 0.0 || record.value > self.validation_threshold {
            return Err(ProcessingError::InvalidValue);
        }

        if record.name.is_empty() || record.category.is_empty() {
            return Err(ProcessingError::MissingField);
        }

        Ok(())
    }

    pub fn transform_category(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        match self.category_mapping.get(&record.category) {
            Some(new_category) => {
                record.category = new_category.clone();
                Ok(())
            }
            None => Err(ProcessingError::CategoryNotFound),
        }
    }

    pub fn normalize_value(&self, record: &mut DataRecord) {
        record.value = record.value / self.validation_threshold;
    }

    pub fn process_records(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());

        for mut record in records {
            self.validate_record(&record)?;
            self.transform_category(&mut record)?;
            self.normalize_value(&mut record);
            processed_records.push(record);
        }

        Ok(processed_records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_mapping() -> HashMap<String, String> {
        let mut mapping = HashMap::new();
        mapping.insert("A".to_string(), "Alpha".to_string());
        mapping.insert("B".to_string(), "Beta".to_string());
        mapping
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(create_test_mapping(), 100.0);
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(create_test_mapping(), 100.0);
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 150.0,
            category: "A".to_string(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_category_transformation() {
        let processor = DataProcessor::new(create_test_mapping(), 100.0);
        let mut record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };

        assert!(processor.transform_category(&mut record).is_ok());
        assert_eq!(record.category, "Alpha");
    }

    #[test]
    fn test_value_normalization() {
        let processor = DataProcessor::new(create_test_mapping(), 100.0);
        let mut record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };

        processor.normalize_value(&mut record);
        assert_eq!(record.value, 0.5);
    }
}