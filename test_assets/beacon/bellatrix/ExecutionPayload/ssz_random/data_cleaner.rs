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

fn clean_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

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
    record.id > 0 && 
    record.category.len() <= 10
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";
    
    clean_data(input_file, output_file)?;
    
    let validation_file = File::open(output_file)?;
    let mut validation_reader = Reader::from_reader(validation_file);
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for result in validation_reader.deserialize() {
        let record: Record = result?;
        
        if validate_record(&record) {
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }
    
    println!("Valid records: {}", valid_count);
    println!("Invalid records: {}", invalid_count);
    
    Ok(())
}