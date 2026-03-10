use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn read_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 4 {
            continue;
        }

        let record = CsvRecord {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            value: parts[2].parse()?,
            category: parts[3].to_string(),
        };

        records.push(record);
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<&CsvRecord> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average_value(records: &[CsvRecord]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let total: f64 = records.iter().map(|record| record.value).sum();
    total / records.len() as f64
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn process_csv_data(file_path: &str) -> Result<(), Box<dyn Error>> {
    let records = read_csv_file(file_path)?;
    
    println!("Total records: {}", records.len());
    
    let filtered = filter_by_category(&records, "premium");
    println!("Premium category records: {}", filtered.len());
    
    let average = calculate_average_value(&records);
    println!("Average value: {:.2}", average);
    
    if let Some(max_record) = find_max_value_record(&records) {
        println!("Max value record: ID={}, Name={}, Value={}", 
                 max_record.id, max_record.name, max_record.value);
    }
    
    Ok(())
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub struct ProcessedRecord {
    id: u32,
    normalized_name: String,
    adjusted_value: f64,
    category_code: u8,
}

pub fn load_csv_records<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);
    
    let mut records = Vec::new();
    for result in csv_reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

pub fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    
    if !record.category.chars().all(|c| c.is_alphabetic()) {
        return Err("Category must contain only letters".to_string());
    }
    
    Ok(())
}

pub fn process_record(record: Record) -> Result<ProcessedRecord, String> {
    validate_record(&record)?;
    
    let normalized_name = record.name.to_uppercase();
    let adjusted_value = record.value * 1.1;
    let category_code = match record.category.as_str() {
        "A" | "ALPHA" => 1,
        "B" | "BETA" => 2,
        "G" | "GAMMA" => 3,
        _ => 0,
    };
    
    Ok(ProcessedRecord {
        id: record.id,
        normalized_name,
        adjusted_value,
        category_code,
    })
}

pub fn save_processed_records<P: AsRef<Path>>(
    records: Vec<ProcessedRecord>,
    path: P,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut csv_writer = csv::Writer::from_writer(writer);
    
    for record in records {
        csv_writer.serialize(record)?;
    }
    
    csv_writer.flush()?;
    Ok(())
}

pub fn process_csv_file(input_path: &str, output_path: &str) -> Result<usize, Box<dyn Error>> {
    let records = load_csv_records(input_path)?;
    let mut processed_records = Vec::new();
    let mut error_count = 0;
    
    for record in records {
        match process_record(record) {
            Ok(processed) => processed_records.push(processed),
            Err(e) => {
                eprintln!("Failed to process record: {}", e);
                error_count += 1;
            }
        }
    }
    
    save_processed_records(processed_records, output_path)?;
    
    Ok(error_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_validate_record_valid() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "Alpha".to_string(),
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_name() {
        let record = Record {
            id: 1,
            name: "   ".to_string(),
            value: 100.0,
            category: "Alpha".to_string(),
        };
        
        assert!(validate_record(&record).is_err());
    }
    
    #[test]
    fn test_process_record() {
        let record = Record {
            id: 42,
            name: "example".to_string(),
            value: 50.0,
            category: "BETA".to_string(),
        };
        
        let processed = process_record(record).unwrap();
        assert_eq!(processed.id, 42);
        assert_eq!(processed.normalized_name, "EXAMPLE");
        assert_eq!(processed.adjusted_value, 55.0);
        assert_eq!(processed.category_code, 2);
    }
    
    #[test]
    fn test_csv_roundtrip() {
        let records = vec![
            Record {
                id: 1,
                name: "First".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            Record {
                id: 2,
                name: "Second".to_string(),
                value: 20.0,
                category: "B".to_string(),
            },
        ];
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        let mut writer = csv::Writer::from_writer(&input_file);
        for record in &records {
            writer.serialize(record).unwrap();
        }
        writer.flush().unwrap();
        
        let error_count = process_csv_file(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        ).unwrap();
        
        assert_eq!(error_count, 0);
    }
}