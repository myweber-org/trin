
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ProcessingError {
    details: String,
}

impl ProcessingError {
    fn new(msg: &str) -> Self {
        ProcessingError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ProcessingError {}

pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: u64) -> Result<Self, ProcessingError> {
        if value < 0.0 || value > 1000.0 {
            return Err(ProcessingError::new("Value must be between 0 and 1000"));
        }
        if timestamp == 0 {
            return Err(ProcessingError::new("Timestamp cannot be zero"));
        }
        
        Ok(DataRecord {
            id,
            value,
            timestamp,
        })
    }
    
    pub fn transform(&self, multiplier: f64) -> Result<f64, ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::new("Multiplier must be positive"));
        }
        
        let transformed = self.value * multiplier;
        if transformed > 5000.0 {
            return Err(ProcessingError::new("Transformed value exceeds maximum limit"));
        }
        
        Ok(transformed)
    }
    
    pub fn validate_consistency(&self, previous_value: Option<f64>) -> Result<bool, ProcessingError> {
        if let Some(prev) = previous_value {
            let change = (self.value - prev).abs();
            if change > 100.0 {
                return Err(ProcessingError::new("Value change exceeds allowed threshold"));
            }
            Ok(change < 50.0)
        } else {
            Ok(true)
        }
    }
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<f64>, ProcessingError> {
    if records.is_empty() {
        return Err(ProcessingError::new("No records to process"));
    }
    
    let mut results = Vec::with_capacity(records.len());
    let mut previous_value: Option<f64> = None;
    
    for record in records {
        record.validate_consistency(previous_value)?;
        
        let transformed = record.transform(2.5)?;
        results.push(transformed);
        
        previous_value = Some(record.value);
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 500.0, 1234567890);
        assert!(record.is_ok());
    }
    
    #[test]
    fn test_invalid_value_record() {
        let record = DataRecord::new(1, -10.0, 1234567890);
        assert!(record.is_err());
    }
    
    #[test]
    fn test_transform_with_valid_multiplier() {
        let record = DataRecord::new(1, 100.0, 1234567890).unwrap();
        let result = record.transform(3.0);
        assert_eq!(result.unwrap(), 300.0);
    }
    
    #[test]
    fn test_process_multiple_records() {
        let records = vec![
            DataRecord::new(1, 100.0, 1234567890).unwrap(),
            DataRecord::new(2, 150.0, 1234567891).unwrap(),
            DataRecord::new(3, 200.0, 1234567892).unwrap(),
        ];
        
        let results = process_records(&records);
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 3);
    }
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }
    
    writer.flush()?;
    
    println!("Processing complete:");
    println!("  Valid records: {}", valid_count);
    println!("  Invalid records: {}", invalid_count);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "input_data.csv";
    let output_file = "processed_data.csv";
    
    match process_csv(input_file, output_file) {
        Ok(_) => println!("Data processing successful"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }
    
    Ok(())
}