use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    email: String,
    age: u8,
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate_age(age: u8) -> Option<u8> {
    if age > 0 && age < 120 {
        Some(age)
    } else {
        None
    }
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
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

    for result in csv_reader.deserialize() {
        let mut record: Record = result?;
        
        record.email = normalize_email(&record.email);
        record.name = record.name.trim().to_string();
        
        if let Some(valid_age) = validate_age(record.age) {
            record.age = valid_age;
            csv_writer.serialize(&record)?;
        }
    }

    csv_writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";
    
    match clean_csv(input_file, output_file) {
        Ok(_) => println!("Data cleaning completed successfully."),
        Err(e) => eprintln!("Error during data cleaning: {}", e),
    }
    
    Ok(())
}