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

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid ID: ID cannot be zero".into());
        }

        if self.values.is_empty() {
            return Err("Invalid data: values vector cannot be empty".into());
        }

        for &value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid data: values contain NaN or infinite numbers".into());
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();

    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.normalize();
        processed_record.add_metadata(
            "processed_timestamp".to_string(),
            chrono::Utc::now().to_rfc3339(),
        );
        processed.push(processed_record);
    }

    Ok(processed)
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
        let mut record = DataRecord::new(1, vec![2.0, 4.0, 6.0]);
        record.normalize();
        assert_eq!(record.values, vec![1.0 / 3.0, 2.0 / 3.0, 1.0]);
    }

    #[test]
    fn test_metadata_addition() {
        let mut record = DataRecord::new(1, vec![1.0]);
        record.add_metadata("source".to_string(), "test".to_string());
        assert_eq!(record.metadata.get("source"), Some(&"test".to_string()));
    }
}