use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut map = std::collections::HashMap::new();
        for record in &self.records {
            map.entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "id,name,value,category\n1,ItemA,25.5,Alpha\n2,ItemB,-10.0,Beta\n3,,15.0,Alpha"
        )
        .unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_csv(file.path()).unwrap();

        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.validate_records().len(), 1);
        assert_eq!(processor.calculate_total(), 30.5);

        let grouped = processor.group_by_category();
        assert_eq!(grouped.get("Alpha").unwrap().len(), 2);
        assert_eq!(grouped.get("Beta").unwrap().len(), 1);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidValue,
    InvalidTimestamp,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "Invalid record ID"),
            ValidationError::InvalidValue => write!(f, "Invalid value field"),
            ValidationError::InvalidTimestamp => write!(f, "Invalid timestamp"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if !record.value.is_finite() {
        return Err(ValidationError::InvalidValue);
    }
    
    if record.timestamp < 0 {
        return Err(ValidationError::InvalidTimestamp);
    }
    
    Ok(())
}

pub fn transform_record(record: &DataRecord) -> DataRecord {
    DataRecord {
        id: record.id,
        value: record.value * 1.1,
        timestamp: record.timestamp + 3600,
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Vec<Result<DataRecord, ValidationError>> {
    records
        .into_iter()
        .map(|record| {
            validate_record(&record)?;
            Ok(transform_record(&record))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1672531200,
        };
        
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1672531200,
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }

    #[test]
    fn test_transform_record() {
        let original = DataRecord {
            id: 1,
            value: 100.0,
            timestamp: 1000,
        };
        
        let transformed = transform_record(&original);
        
        assert_eq!(transformed.id, 1);
        assert_eq!(transformed.value, 110.0);
        assert_eq!(transformed.timestamp, 4600);
    }
}