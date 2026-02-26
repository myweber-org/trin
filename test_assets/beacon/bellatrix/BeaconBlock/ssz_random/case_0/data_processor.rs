use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    fn new(id: u32, value: f64, category: &str) -> Result<Self, ValidationError> {
        if value < 0.0 || value > 1000.0 {
            return Err(ValidationError {
                message: format!("Value {} out of range [0, 1000]", value),
            });
        }
        
        if category.is_empty() {
            return Err(ValidationError {
                message: "Category cannot be empty".to_string(),
            });
        }
        
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
    }
    
    fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
    
    fn display(&self) -> String {
        format!("ID: {}, Value: {:.2}, Category: {}", 
                self.id, self.value, self.category)
    }
}

fn process_records(records: &mut [DataRecord]) -> Vec<String> {
    records.iter_mut()
        .map(|record| {
            record.transform(1.05);
            record.display()
        })
        .collect()
}

fn validate_and_create_records(data: Vec<(u32, f64, &str)>) -> Result<Vec<DataRecord>, ValidationError> {
    let mut records = Vec::new();
    
    for (id, value, category) in data {
        let record = DataRecord::new(id, value, category)?;
        records.push(record);
    }
    
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, "analytics").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 500.0);
        assert_eq!(record.category, "analytics");
    }
    
    #[test]
    fn test_invalid_value() {
        let result = DataRecord::new(2, 1500.0, "analytics");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(3, 200.0, "metrics").unwrap();
        record.transform(2.0);
        assert_eq!(record.value, 400.0);
    }
}