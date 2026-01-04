
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
    category_totals: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_totals: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.category_totals
            .entry(record.category.clone())
            .and_modify(|total| *total += record.value)
            .or_insert(record.value);

        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_total(&self, category: &str) -> f64 {
        *self.category_totals.get(category).unwrap_or(&0.0)
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.records.values().map(|r| r.value).sum::<f64>() / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }

        self.recalculate_totals();
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.value < 0.0 || record.value > 1000.0 {
            return Err(DataError::InvalidValue);
        }

        if record.category.trim().is_empty() {
            return Err(DataError::InvalidCategory);
        }

        Ok(())
    }

    fn recalculate_totals(&mut self) {
        self.category_totals.clear();
        for record in self.records.values() {
            *self.category_totals
                .entry(record.category.clone())
                .or_insert(0.0) += record.value;
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
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: "Test1".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        let record2 = DataRecord {
            id: 1,
            name: "Test2".to_string(),
            value: 200.0,
            category: "B".to_string(),
        };

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
    }

    #[test]
    fn test_category_totals() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Item1".to_string(),
                value: 50.0,
                category: "Electronics".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Item2".to_string(),
                value: 75.0,
                category: "Electronics".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Item3".to_string(),
                value: 100.0,
                category: "Books".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        assert_eq!(processor.get_category_total("Electronics"), 125.0);
        assert_eq!(processor.get_category_total("Books"), 100.0);
        assert_eq!(processor.calculate_average(), 75.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        processor.add_record(record).unwrap();
        processor.transform_values(|x| x * 1.1);

        let updated_record = processor.get_record(1).unwrap();
        assert_eq!(updated_record.value, 110.0);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        }
    }

    pub fn add_validator(&mut self, name: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn add_transformer(&mut self, name: &str, transformer: Box<dyn Fn(String) -> String>) {
        self.transformers.insert(name.to_string(), transformer);
    }

    pub fn validate(&self, name: &str, value: &str) -> bool {
        match self.validators.get(name) {
            Some(validator) => validator(value),
            None => false,
        }
    }

    pub fn transform(&self, name: &str, value: String) -> Option<String> {
        self.transformers.get(name).map(|transformer| transformer(value))
    }

    pub fn process_data(&self, value: &str) -> Option<String> {
        if !self.validate("email", value) {
            return None;
        }

        let transformed = self.transform("uppercase", value.to_string())?;
        Some(transformed)
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validator("email", Box::new(|value| {
        value.contains('@') && value.contains('.')
    }));

    processor.add_transformer("uppercase", Box::new(|value| {
        value.to_uppercase()
    }));

    processor.add_transformer("trim", Box::new(|value| {
        value.trim().to_string()
    }));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let processor = create_default_processor();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }

    #[test]
    fn test_uppercase_transformation() {
        let processor = create_default_processor();
        let result = processor.transform("uppercase", "hello".to_string());
        assert_eq!(result, Some("HELLO".to_string()));
    }

    #[test]
    fn test_process_data() {
        let processor = create_default_processor();
        let result = processor.process_data("user@domain.com");
        assert_eq!(result, Some("USER@DOMAIN.COM".to_string()));
    }
}