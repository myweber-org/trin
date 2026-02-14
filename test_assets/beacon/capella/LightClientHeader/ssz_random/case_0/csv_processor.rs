use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    Ok(())
}

fn process_csv(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        match validate_record(&record) {
            Ok(_) => {
                writer.serialize(&record)?;
                println!("Processed valid record: {:?}", record);
            }
            Err(err) => {
                eprintln!("Invalid record skipped: {} - Error: {}", record.id, err);
            }
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = Path::new("data/input.csv");
    let output_path = Path::new("data/output.csv");
    
    process_csv(input_path, output_path)?;
    
    println!("CSV processing completed successfully");
    Ok(())
}