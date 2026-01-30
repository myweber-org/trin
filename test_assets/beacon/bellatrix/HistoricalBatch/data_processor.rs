
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
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::DuplicateTag => write!(f, "Duplicate tags are not allowed"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &self.tags {
            if !seen_tags.insert(tag) {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) -> &mut Self {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
        self.tags.sort();
        self.tags.dedup();
        self
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        record.validate()?;
        
        if self.records.contains_key(&record.id) {
            return Err(format!("Record with ID {} already exists", record.id).into());
        }
        
        self.records.insert(record.id, record);
        Ok(())
    }
    
    pub fn process_records(&mut self, multiplier: f64) -> Vec<DataRecord> {
        let mut processed = Vec::new();
        
        for (_, record) in &mut self.records {
            let mut transformed = record.clone();
            transformed.transform(multiplier);
            processed.push(transformed);
        }
        
        processed.sort_by(|a, b| a.id.cmp(&b.id));
        processed
    }
    
    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let values: Vec<f64> = self.records.values().map(|r| r.value).collect();
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;
        
        let max = values.iter().fold(f64::MIN, |a, &b| a.max(b));
        
        (mean, variance.sqrt(), max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -5.0,
            tags: vec!["tag1".to_string(), "tag1".to_string()],
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: 10.0,
            tags: vec!["z".to_string(), "a".to_string(), "z".to_string()],
        };
        
        record.transform(2.0);
        
        assert_eq!(record.name, "TEST");
        assert_eq!(record.value, 20.0);
        assert_eq!(record.tags, vec!["a".to_string(), "z".to_string()]);
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 10.0,
            tags: vec!["alpha".to_string()],
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Second".to_string(),
            value: 20.0,
            tags: vec!["beta".to_string()],
        };
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        let processed = processor.process_records(1.5);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].value, 15.0);
        assert_eq!(processed[1].value, 30.0);
        
        let (mean, std_dev, max) = processor.get_statistics();
        assert!((mean - 15.0).abs() < 0.001);
        assert!((std_dev - 7.5).abs() < 0.001);
        assert_eq!(max, 30.0);
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

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn validate_records(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for (index, record) in self.records.iter().enumerate() {
            if record.name.trim().is_empty() {
                errors.push(format!("Record {}: Name is empty", index));
            }

            if record.value < 0.0 {
                errors.push(format!("Record {}: Value is negative", index));
            }

            if record.category.trim().is_empty() {
                errors.push(format!("Record {}: Category is empty", index));
            }
        }

        errors
    }

    pub fn get_statistics(&self) -> String {
        let count = self.records.len();
        let avg = self.calculate_average().unwrap_or(0.0);
        let max_record = self.find_max_value();
        let validation_errors = self.validate_records();

        let mut stats = format!(
            "Total records: {}\nAverage value: {:.2}\n",
            count, avg
        );

        if let Some(record) = max_record {
            stats.push_str(&format!(
                "Maximum value: {} (ID: {}, Name: {})\n",
                record.value, record.id, record.name
            ));
        }

        if !validation_errors.is_empty() {
            stats.push_str(&format!("Validation errors: {}\n", validation_errors.len()));
            for error in validation_errors.iter().take(3) {
                stats.push_str(&format!("  - {}\n", error));
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item A,10.5,Electronics").unwrap();
        writeln!(temp_file, "2,Item B,25.0,Electronics").unwrap();
        writeln!(temp_file, "3,Item C,15.75,Furniture").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);

        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 17.0833).abs() < 0.001);

        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);

        let errors = processor.validate_records();
        assert!(errors.is_empty());
    }
}