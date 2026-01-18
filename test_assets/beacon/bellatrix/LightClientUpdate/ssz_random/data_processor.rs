
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
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    InvalidCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than zero"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.name.trim().is_empty() {
        return Err(ValidationError::EmptyName);
    }
    
    if record.value < 0.0 {
        return Err(ValidationError::NegativeValue);
    }
    
    let valid_categories = ["A", "B", "C"];
    if !valid_categories.contains(&record.category.as_str()) {
        return Err(ValidationError::InvalidCategory);
    }
    
    Ok(())
}

pub fn process_records(records: Vec<DataRecord>) -> Result<HashMap<String, Vec<DataRecord>>, Box<dyn Error>> {
    let mut categorized_records: HashMap<String, Vec<DataRecord>> = HashMap::new();
    
    for record in records {
        validate_record(&record)?;
        
        categorized_records
            .entry(record.category.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    Ok(categorized_records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

pub fn transform_records(records: Vec<DataRecord>, multiplier: f64) -> Vec<DataRecord> {
    records.into_iter()
        .map(|mut record| {
            record.value *= multiplier;
            record
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record1".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record2".to_string(),
                value: 20.0,
                category: "B".to_string(),
            },
        ];
        
        let result = process_records(records).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("A").unwrap().len(), 1);
        assert_eq!(result.get("B").unwrap().len(), 1);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "R1".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "R2".to_string(),
                value: 20.0,
                category: "A".to_string(),
            },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 15.0);
        assert_eq!(variance, 25.0);
        assert_eq!(std_dev, 5.0);
    }
    
    #[test]
    fn test_transform_records() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "R1".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
        ];
        
        let transformed = transform_records(records, 2.0);
        assert_eq!(transformed[0].value, 20.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let mut count = 0;
        for result in rdr.records() {
            let record = result?;
            if record.len() >= 3 {
                let id: u32 = record[0].parse().unwrap_or(0);
                let value: f64 = record[1].parse().unwrap_or(0.0);
                let category = record[2].to_string();
                let valid = value > 0.0 && !category.is_empty();
                
                self.records.push(DataRecord {
                    id,
                    value,
                    category,
                    valid,
                });
                count += 1;
            }
        }
        
        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        if valid_records.is_empty() {
            return None;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn get_categories(&self) -> Vec<String> {
        let mut categories = Vec::new();
        for record in &self.records {
            if !categories.contains(&record.category) {
                categories.push(record.category.clone());
            }
        }
        categories.sort();
        categories
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,25.5,TypeA").unwrap();
        writeln!(temp_file, "2,0.0,TypeB").unwrap();
        writeln!(temp_file, "3,42.8,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 34.15).abs() < 0.01);
        
        let categories = processor.get_categories();
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0], "TypeA");
    }
}