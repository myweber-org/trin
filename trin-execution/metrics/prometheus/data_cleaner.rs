use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u32,
    email: String,
}

fn clean_csv_data(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.email = record.email.trim().to_lowercase();
        
        if record.age > 150 {
            record.age = 150;
        }
        
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() 
        && record.age > 0 
        && record.email.contains('@')
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = Path::new("input.csv");
    let output_path = Path::new("cleaned_output.csv");
    
    clean_csv_data(input_path, output_path)?;
    
    let validation_file = File::open(output_path)?;
    let mut validation_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(validation_file);
    
    let mut valid_count = 0;
    let mut total_count = 0;
    
    for result in validation_reader.deserialize() {
        let record: Record = result?;
        total_count += 1;
        
        if validate_record(&record) {
            valid_count += 1;
        }
    }
    
    println!("Processed {} records, {} valid", total_count, valid_count);
    Ok(())
}