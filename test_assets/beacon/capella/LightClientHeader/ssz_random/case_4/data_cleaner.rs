use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = Reader::from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let mut csv_writer = Writer::from_writer(output_file);

    let mut cleaned_count = 0;
    let mut error_count = 0;

    for result in csv_reader.deserialize() {
        match result {
            Ok(mut record) => {
                let rec: Record = record;
                let cleaned_record = clean_record(rec);
                csv_writer.serialize(&cleaned_record)?;
                cleaned_count += 1;
            }
            Err(e) => {
                eprintln!("Error parsing record: {}", e);
                error_count += 1;
            }
        }
    }

    csv_writer.flush()?;
    
    println!("Data cleaning completed:");
    println!("  Cleaned records: {}", cleaned_count);
    println!("  Error records: {}", error_count);
    
    Ok(())
}

fn clean_record(mut record: Record) -> Record {
    record.name = record.name.trim().to_string();
    if record.name.is_empty() {
        record.name = "Unknown".to_string();
    }
    
    record.category = record.category.to_lowercase();
    
    if record.value < 0.0 {
        record.value = 0.0;
    } else if record.value > 1000.0 {
        record.value = 1000.0;
    }
    
    record
}

fn validate_file_path(path: &str) -> bool {
    Path::new(path).exists()
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/raw_data.csv";
    let output_file = "data/cleaned_data.csv";
    
    if !validate_file_path(input_file) {
        eprintln!("Input file does not exist: {}", input_file);
        return Ok(());
    }
    
    match clean_csv_data(input_file, output_file) {
        Ok(_) => println!("Successfully cleaned data to {}", output_file),
        Err(e) => eprintln!("Error cleaning data: {}", e),
    }
    
    Ok(())
}