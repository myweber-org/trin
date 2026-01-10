use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_uppercase();
        
        if record.value < 0.0 {
            record.value = 0.0;
        }
        
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && 
    record.value >= 0.0 && 
    !record.category.is_empty()
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "data/raw.csv";
    let output = "data/cleaned.csv";
    
    match clean_csv_data(input, output) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error during data cleaning: {}", e),
    }
    
    Ok(())
}