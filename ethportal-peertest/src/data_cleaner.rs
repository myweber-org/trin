use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    email: String,
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.age > 120 {
        return Err("Age must be reasonable".to_string());
    }
    if !record.email.contains('@') {
        return Err("Invalid email format".to_string());
    }
    Ok(())
}

fn clean_data(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        match validate_record(&record) {
            Ok(_) => {
                writer.serialize(&record)?;
                println!("Valid record: {:?}", record);
            }
            Err(e) => {
                eprintln!("Invalid record {}: {}", record.id, e);
            }
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = Path::new("input.csv");
    let output = Path::new("cleaned.csv");
    
    clean_data(input, output)?;
    println!("Data cleaning completed successfully");
    Ok(())
}