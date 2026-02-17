
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use log::{info, warn};

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug)]
struct CleanedRecord {
    id: u32,
    name: String,
    value: f64,
    category: String,
    is_valid: bool,
}

impl CleanedRecord {
    fn new(record: Record) -> Self {
        let is_valid = !record.name.is_empty() 
            && record.value >= 0.0 
            && !record.category.is_empty();
        
        CleanedRecord {
            id: record.id,
            name: record.name.trim().to_string(),
            value: record.value,
            category: record.category.trim().to_string(),
            is_valid,
        }
    }
}

pub fn clean_csv_data(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    info!("Starting CSV data cleaning process");
    
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for result in csv_reader.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to parse record: {}", e);
                invalid_count += 1;
                continue;
            }
        };
        
        let cleaned_record = CleanedRecord::new(record);
        
        if cleaned_record.is_valid {
            csv_writer.serialize(&cleaned_record)?;
            valid_count += 1;
        } else {
            warn!("Invalid record detected: ID {}", cleaned_record.id);
            invalid_count += 1;
        }
    }
    
    csv_writer.flush()?;
    
    info!("Data cleaning completed. Valid records: {}, Invalid records: {}", 
          valid_count, invalid_count);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_clean_csv_data() {
        let input_data = "id,name,value,category\n1,Test,42.5,CategoryA\n2,,15.0,CategoryB\n";
        let mut input_file = NamedTempFile::new().unwrap();
        std::fs::write(input_file.path(), input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = clean_csv_data(input_file.path(), output_file.path());
        assert!(result.is_ok());
        
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("Test"));
        assert!(!output_content.contains("CategoryB"));
    }
}