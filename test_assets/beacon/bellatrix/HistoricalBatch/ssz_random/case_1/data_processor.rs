use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        self.records.clear();

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn save_filtered_results(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in filtered {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.records
            .iter()
            .map(|record| record.value)
            .fold(f64::NEG_INFINITY, f64::max);

        (count, avg, max)
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,15.2,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,8.7,Category1").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        processor.load_from_csv(file_path).unwrap();
        
        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);
        
        let avg = processor.calculate_average();
        assert!((avg - 11.466666666666667).abs() < 0.0001);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
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

pub fn transform_records(records: Vec<DataRecord>) -> HashMap<String, Vec<DataRecord>> {
    let mut grouped = HashMap::new();
    
    for record in records {
        grouped
            .entry(record.category.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    grouped
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
    fn test_transform_records() {
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 20.0, category: "B".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 30.0, category: "A".to_string() },
        ];
        
        let grouped = transform_records(records);
        
        assert_eq!(grouped.get("A").unwrap().len(), 2);
        assert_eq!(grouped.get("B").unwrap().len(), 1);
        assert!(grouped.get("C").is_none());
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 20.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 30.0, category: "A".to_string() },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}