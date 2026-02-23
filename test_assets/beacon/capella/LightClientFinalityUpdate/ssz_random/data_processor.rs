
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
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        let mut count = 0;
        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, "Item1".to_string(), 10.0, "CategoryA".to_string()));
        processor.add_record(DataRecord::new(2, "Item2".to_string(), 20.0, "CategoryB".to_string()));
        
        assert_eq!(processor.get_records().len(), 2);
        assert_eq!(processor.calculate_average(), Some(15.0));
        
        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Item1");
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, "Test".to_string(), 100.0, "Demo".to_string()));
        
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();
        
        processor.save_to_csv(path)?;
        
        let mut new_processor = DataProcessor::new();
        let count = new_processor.load_from_csv(path)?;
        
        assert_eq!(count, 1);
        assert_eq!(new_processor.get_records().len(), 1);
        
        Ok(())
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        if data.is_empty() {
            return Err("No data available for statistics".into());
        }

        let mut values = Vec::new();
        for record in data {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }
            
            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
            } else {
                return Err(format!("Cannot parse value '{}' as number", record[column_index]).into());
            }
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        Ok((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], vec!["Alice", "30", "50000.0"]);
    }

    #[test]
    fn test_statistics_calculation() {
        let data = vec![
            vec!["50000.0".to_string()],
            vec!["45000.0".to_string()],
            vec!["55000.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&data, 0);
        
        assert!(stats.is_ok());
        let (mean, std_dev) = stats.unwrap();
        assert!((mean - 50000.0).abs() < 0.01);
        assert!(std_dev > 0.0);
    }
}